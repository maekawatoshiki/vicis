use rustc_hash::FxHashSet;
use vicis_core::{
    // codegen::{isa::x86_64::X86_64, lower::compile_module},
    // exec::{generic_value::GenericValue, interpreter::Interpreter},
    ir::{function::basic_block::BasicBlockId, module},
    pass::analysis::dom_tree::DominatorTree,
};

#[test]
fn dom1() {
    let src = r#"
define dso_local i32 @main() {
  %1 = alloca i32, align 4
  %2 = alloca i32, align 4
  %3 = alloca i32, align 4
  store i32 0, i32* %1, align 4
  store i32 1, i32* %2, align 4
  %4 = load i32, i32* %2, align 4
  %5 = icmp slt i32 %4, 2
  br i1 %5, label %6, label %11

6:                                                ; preds = %0
  store i32 10, i32* %3, align 4
  %7 = load i32, i32* %2, align 4
  %8 = icmp slt i32 %7, 5
  br i1 %8, label %9, label %10

9:                                                ; preds = %6
  store i32 30, i32* %3, align 4
  br label %10

10:                                               ; preds = %9, %6
  br label %12

11:                                               ; preds = %0
  store i32 20, i32* %3, align 4
  br label %12

12:                                               ; preds = %11, %10
  %13 = load i32, i32* %3, align 4
  ret i32 %13
}
        "#;

    let module = module::parse_assembly(src).unwrap();

    for (_, func) in module.functions() {
        let blocks: Vec<BasicBlockId> = func.data.basic_blocks.iter().map(|(id, _)| id).collect();
        let dom_tree = DominatorTree::new(func);

        assert!(dom_tree.dominates(blocks[1], blocks[1]));
        assert!(dom_tree.dominates(blocks[1], blocks[2]));
        assert!(dom_tree.dominates(blocks[1], blocks[3]));
        assert!(dom_tree.dominates(blocks[1], blocks[4]));
        assert!(dom_tree.dominates(blocks[1], blocks[5]));
        assert!(dom_tree.dominates(blocks[1], blocks[6]));
        assert!(dom_tree.dominates(blocks[2], blocks[4]));
        assert!(dom_tree.dominates(blocks[2], blocks[5]));
        assert!(!dom_tree.dominates(blocks[3], blocks[5]));
        assert!(!dom_tree.dominates(blocks[3], blocks[6]));
        assert!(
            dom_tree.dominance_frontier_of(blocks[5])
                == Some(&vec![blocks[6]].into_iter().collect::<FxHashSet<_>>())
        );
        assert!(
            dom_tree.dominance_frontier_of(blocks[2])
                == Some(&vec![blocks[6]].into_iter().collect::<FxHashSet<_>>())
        );
        assert!(
            dom_tree.dominance_frontier_of(blocks[4])
                == Some(&vec![blocks[5]].into_iter().collect::<FxHashSet<_>>())
        );
        assert!(
            dom_tree.dominance_frontier_of(blocks[3])
                == Some(&vec![blocks[6]].into_iter().collect::<FxHashSet<_>>())
        );
        assert!(
            dom_tree.dominance_frontier_of(blocks[1])
                == Some(&vec![].into_iter().collect::<FxHashSet<_>>())
        );
        assert!(
            dom_tree.dominance_frontier_of(blocks[6])
                == Some(&vec![].into_iter().collect::<FxHashSet<_>>())
        );
    }
}
