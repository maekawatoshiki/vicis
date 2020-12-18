pub mod parser;

pub use parser::parse;

use super::types::{TypeId, Types};

pub struct Function {
    pub name: String,
    pub is_var_arg: bool,
    pub result_ty: TypeId,
    pub params: Vec<Parameter>,
    pub data: Data,
    pub layout: Layout,
    pub types: Types,
    // data: FunctionData,
    // body {
    //     // Data
    //     {
    //         basicblock_definitions,
    //         instruction_defintions,
    //     }
    //     // Layout
    //     {
    //         basicblock_ordering,
    //         instruction_ordering,
    //     }
    // }
}

pub struct Parameter {}

pub struct Data {
    // instructions: Arena<Instruction>,
// basic_blocks: Arena<BasicBlock>,
}

pub struct Layout {
    // basic_blocks: FxHashMap<BasicBlockId, BasicBlockNode>,
// instructions: FxHashMap<InstructionId, InstructionNode>,
}

pub struct BasicBlockNode {
    // prev: Option<BasicBlockId>,
// next: Option<BasicBlockId>,
// first_inst: Option<InstructionId>,
// last_inst: Option<InstructionId>,
}

pub struct InstructionNode {
    // prev: Option<InstructionId>,
// next: Option<InstructionId>,
}

impl Data {
    pub fn new() -> Self {
        Self {}
    }
}

impl Layout {
    pub fn new() -> Self {
        Self {}
    }
}
