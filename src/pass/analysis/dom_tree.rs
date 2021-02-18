use id_arena::Id;
use rustc_hash::{FxHashMap, FxHashSet};
use std::fmt;

#[derive(Debug)]
pub struct DominatorTree<BB: BasicBlock> {
    dom: DomTree<BB>,
    frontier: FxHashMap<Id<BB>, FxHashSet<Id<BB>>>,
    level: FxHashMap<Id<BB>, usize>,
    root: Id<BB>,
}

type DomTree<BB> = FxHashMap<Id<BB>, FxHashSet<Id<BB>>>;

pub trait BasicBlock: Sized + fmt::Debug {
    fn preds(&self) -> &FxHashSet<Id<Self>>;
    fn succs(&self) -> &FxHashSet<Id<Self>>;
}

pub trait BasicBlockData<BB: BasicBlock> {
    fn get(&self, id: Id<BB>) -> &BB;
}

pub trait BasicBlockLayout<BB: BasicBlock> {
    fn order(&self) -> Box<dyn Iterator<Item = Id<BB>> + '_>;
}

type Map<T> = FxHashMap<T, T>;

struct Context<'a, BB: BasicBlock, F: BasicBlockData<BB> + BasicBlockLayout<BB>> {
    f: &'a F,
    dfnum: FxHashMap<Id<BB>, usize>,
    vertex: Vec<Id<BB>>,
    semi: Map<Id<BB>>,
    ancestor: Map<Id<BB>>,
    idom: Map<Id<BB>>,
    samedom: Map<Id<BB>>,
    parent: Map<Id<BB>>,
    best: Map<Id<BB>>,
}

impl<BB: BasicBlock> DominatorTree<BB> {
    pub fn new<F: BasicBlockData<BB> + BasicBlockLayout<BB>>(f: &F) -> Self {
        let mut dom = FxHashMap::default();
        let ctx = Context::new(f).compute();

        // x dominates y
        for (&y, &x) in &ctx.idom {
            dom.entry(x).or_insert(FxHashSet::default()).insert(y);
        }

        fn leveling<BB: BasicBlock>(
            level: &mut FxHashMap<Id<BB>, usize>,
            dom: &FxHashMap<Id<BB>, FxHashSet<Id<BB>>>,
            cur: Id<BB>,
            cur_level: usize,
        ) {
            level.insert(cur, cur_level);
            if dom.get(&cur).is_none() {
                return;
            }
            for &child in dom.get(&cur).unwrap() {
                leveling(level, dom, child, cur_level + 1);
            }
        }

        let entry = f.order().next().unwrap();
        let frontier = Self::compute_dom_frontier(&ctx, &dom, entry);
        let mut level = FxHashMap::default();
        leveling(&mut level, &dom, entry, 0);

        Self {
            dom,
            frontier,
            level,
            root: entry,
        }
    }

    pub fn dominates(&self, x: Id<BB>, y: Id<BB>) -> bool {
        x == y
            || self.dom.get(&x).map_or(false, |children| {
                children.contains(&y) || children.iter().any(|&child| self.dominates(child, y))
            })
    }

    pub fn dominance_frontier_of(&self, x: Id<BB>) -> Option<&FxHashSet<Id<BB>>> {
        self.frontier.get(&x)
    }

    pub fn level_of(&self, x: Id<BB>) -> Option<usize> {
        self.level.get(&x).map(|x| *x)
    }

    pub fn root(&self) -> &Id<BB> {
        &self.root
    }

    fn compute_dom_frontier_of<F: BasicBlockData<BB> + BasicBlockLayout<BB>>(
        ctx: &Context<BB, F>,
        dom: &FxHashMap<Id<BB>, FxHashSet<Id<BB>>>,
        x: Id<BB>,
        frontier: &mut FxHashMap<Id<BB>, FxHashSet<Id<BB>>>,
    ) {
        if frontier.contains_key(&x) {
            // dominance frontier for x is already computed
            return;
        }

        frontier.insert(x, FxHashSet::default());

        for succ in ctx.f.get(x).succs() {
            if ctx.idom.get(succ).map_or(true, |&x_| x != x_) {
                frontier.get_mut(&x).unwrap().insert(*succ);
            }
            for child in dom.get(&x).unwrap_or(&FxHashSet::default()) {
                Self::compute_dom_frontier_of(ctx, dom, *child, frontier);
                for y in frontier.get(child).unwrap().clone() {
                    if ctx.idom.get(&y).map_or(true, |&x_| x_ != x) {
                        frontier.get_mut(&x).unwrap().insert(y);
                    }
                }
            }
        }
    }

    fn compute_dom_frontier<F: BasicBlockData<BB> + BasicBlockLayout<BB>>(
        ctx: &Context<BB, F>,
        dom: &FxHashMap<Id<BB>, FxHashSet<Id<BB>>>,
        start: Id<BB>,
    ) -> FxHashMap<Id<BB>, FxHashSet<Id<BB>>> {
        let mut frontier = FxHashMap::default();
        for &child in dom.get(&start).unwrap_or(&FxHashSet::default()) {
            Self::compute_dom_frontier_of(ctx, dom, child, &mut frontier);
        }
        Self::compute_dom_frontier_of(ctx, dom, start, &mut frontier);
        frontier
    }
}

impl<'a, BB: BasicBlock, F: BasicBlockData<BB> + BasicBlockLayout<BB>> Context<'a, BB, F> {
    fn new(f: &'a F) -> Self {
        Self {
            f,
            dfnum: FxHashMap::default(),
            semi: FxHashMap::default(),
            ancestor: FxHashMap::default(),
            idom: FxHashMap::default(),
            samedom: FxHashMap::default(),
            vertex: Vec::new(),
            parent: FxHashMap::default(),
            best: FxHashMap::default(),
        }
    }

    fn compute(mut self) -> Self {
        let entry = self.f.order().next().unwrap();
        let mut bucket = FxHashMap::default();
        let mut num = 0;

        self.number_by_dfs(None, entry, &mut num);

        for i in (1..num).rev() {
            let node = self.vertex[i];
            let pred = *self.parent.get(&node).unwrap();
            let mut s = pred;

            for v in self.f.get(node).preds() {
                let s_ = if self.dfnum[v] <= self.dfnum[&node] {
                    *v
                } else {
                    let n = self.ancestor_with_lowest_semi(*v);
                    *self.semi.get(&n).unwrap()
                };
                if self.dfnum[&s_] < self.dfnum[&s] {
                    s = s_;
                }
            }

            self.semi.insert(node, s);
            bucket.entry(s).or_insert(FxHashSet::default()).insert(node);
            self.link(pred, node);

            if let Some(set) = bucket.get_mut(&pred) {
                for v in &*set {
                    let y = self.ancestor_with_lowest_semi(*v);
                    if self.semi[&y] == self.semi[&v] {
                        self.idom.insert(*v, pred);
                    } else {
                        self.samedom.insert(*v, y);
                    }
                }
                set.clear();
            }
        }

        for &n in &self.vertex[1..] {
            if let Some(s) = self.samedom.get(&n) {
                self.idom.insert(n, *s);
            }
        }

        self
    }

    fn number_by_dfs(&mut self, pred: Option<Id<BB>>, node: Id<BB>, num: &mut usize) {
        if self.dfnum.contains_key(&node) {
            return;
        }

        self.dfnum.insert(node, *num);
        self.vertex.insert(*num, node);
        if let Some(pred) = pred {
            self.parent.insert(node, pred);
        }
        *num += 1;

        for succ in self.f.get(node).succs() {
            self.number_by_dfs(Some(node), *succ, num);
        }
    }

    fn ancestor_with_lowest_semi(&mut self, node: Id<BB>) -> Id<BB> {
        let a = *self.ancestor.get(&node).unwrap();
        if self.ancestor.contains_key(&a) {
            let b = self.ancestor_with_lowest_semi(a);
            let aa = *self.ancestor.get(&a).unwrap();
            self.ancestor.insert(node, aa);
            if self.dfnum[&self.semi[&b]] < self.dfnum[&self.semi[&self.best[&node]]] {
                self.best.insert(node, b);
            }
        }
        *self.best.get(&node).unwrap()
    }

    fn link(&mut self, pred: Id<BB>, node: Id<BB>) {
        self.ancestor.insert(node, pred);
        self.best.insert(node, node);
    }
}

////

use crate::ir::function::{basic_block::BasicBlock as IrBasicBlock, Function};

impl BasicBlock for IrBasicBlock {
    fn preds(&self) -> &FxHashSet<Id<Self>> {
        &self.preds
    }

    fn succs(&self) -> &FxHashSet<Id<Self>> {
        &self.succs
    }
}

impl BasicBlockData<IrBasicBlock> for Function {
    fn get(&self, id: Id<IrBasicBlock>) -> &IrBasicBlock {
        &self.data.basic_blocks[id]
    }
}

impl BasicBlockLayout<IrBasicBlock> for Function {
    fn order(&self) -> Box<dyn Iterator<Item = Id<IrBasicBlock>> + '_> {
        Box::new(self.layout.block_iter())
    }
}
