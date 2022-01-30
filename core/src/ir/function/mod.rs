pub mod basic_block;
pub mod builder;
pub mod data;
pub mod instruction;
pub mod layout;
pub mod param_attrs;
pub mod print;

use super::{
    module::{
        attributes::Attribute, linkage::Linkage, name::Name,
        preemption_specifier::PreemptionSpecifier, unnamed_addr::UnnamedAddr,
        visibility::Visibility,
    },
    types::{Type, Types},
    value::ConstantData,
};
use crate::traits::basic_block::{BasicBlockData, BasicBlockLayout};
use basic_block::BasicBlock;
use id_arena::Id;
use instruction::InstructionId;
use param_attrs::ParameterAttribute;
use std::fmt;

pub type FunctionId = Id<Function>;

pub type PersonalityFunc = (Type, ConstantData);

pub struct Function {
    pub name: String,
    pub is_var_arg: bool,
    pub result_ty: Type,
    pub params: Vec<Parameter>,
    pub linkage: Linkage,
    pub preemption_specifier: PreemptionSpecifier,
    pub visibility: Visibility,
    pub unnamed_addr: Option<UnnamedAddr>,
    pub func_attrs: Vec<Attribute>,
    pub ret_attrs: Vec<param_attrs::ParameterAttribute>,
    pub personality: Option<PersonalityFunc>,
    pub data: data::Data,
    pub layout: layout::Layout,
    pub types: Types,
    // pub is_prototype: bool,
}

#[derive(Debug, Clone)]
pub struct Parameter {
    pub name: Name,
    pub ty: Type,
    pub attrs: Vec<ParameterAttribute>,
}

impl Function {
    pub fn new<T: AsRef<str>>(
        name: T,
        result_ty: Type,
        params: Vec<Parameter>,
        is_var_arg: bool,
        types: Types,
    ) -> Self {
        Self {
            name: name.as_ref().to_string(),
            is_var_arg,
            result_ty,
            params,
            linkage: Linkage::Common,
            preemption_specifier: PreemptionSpecifier::DsoLocal,
            visibility: Visibility::Default,
            unnamed_addr: None,
            func_attrs: vec![],
            ret_attrs: vec![],
            personality: None,
            data: data::Data::default(),
            layout: layout::Layout::default(),
            types,
        }
    }

    pub fn name(&self) -> &String {
        &self.name
    }

    pub fn params(&self) -> &[Parameter] {
        &self.params
    }

    pub fn is_var_arg(&self) -> bool {
        self.is_var_arg
    }

    pub fn is_prototype(&self) -> bool {
        self.layout.is_empty()
    }

    pub fn remove_inst(&mut self, inst: InstructionId) -> Option<()> {
        self.data.remove_uses(inst);
        self.layout.remove_inst(inst)
    }
}

impl fmt::Debug for Function {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        print::FunctionAsmPrinter::new(f).print(self)
    }
}

impl Parameter {
    pub fn new(ty: Type) -> Self {
        Self {
            name: Name::Number(0),
            ty,
            attrs: vec![],
        }
    }

    pub fn to_string(&self, types: &Types) -> String {
        format!("{} %{:?}", types.to_string(self.ty), self.name)
    }
}

impl BasicBlockData<BasicBlock> for Function {
    fn get(&self, id: Id<BasicBlock>) -> &BasicBlock {
        &self.data.basic_blocks[id]
    }
}

impl BasicBlockLayout<BasicBlock> for Function {
    fn order(&self) -> Box<dyn Iterator<Item = Id<BasicBlock>> + '_> {
        Box::new(self.layout.block_iter())
    }
}
