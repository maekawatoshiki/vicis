use super::{Instruction, InstructionId};
use crate::ir::{
    function::{
        basic_block::BasicBlockId,
        data::Data,
        instruction::{
            Alloca, Cast, ExtractValue, ICmp, InsertValue, IntBinary, Load, Operand, Phi, Store,
        },
    },
    module::name::Name,
    types::Types,
    value::{Value, ValueId},
};
use std::fmt;

pub struct DisplayInstruction<'a> {
    pub inst: &'a Instruction,
    pub data: &'a Data,
    pub types: &'a Types,
    pub inst_name_fn: Option<Box<dyn Fn(InstructionId) -> Option<Name> + 'a>>,
    pub block_name_fn: Option<Box<dyn Fn(BasicBlockId) -> Option<Name> + 'a>>,
}

impl<'a> DisplayInstruction<'a> {
    pub fn set_inst_name_fn(
        mut self,
        name_fn: Box<dyn Fn(InstructionId) -> Option<Name> + 'a>,
    ) -> Self {
        self.inst_name_fn = Some(name_fn);
        self
    }

    pub fn set_block_name_fn(
        mut self,
        name_fn: Box<dyn Fn(BasicBlockId) -> Option<Name> + 'a>,
    ) -> Self {
        self.block_name_fn = Some(name_fn);
        self
    }
}

impl fmt::Display for DisplayInstruction<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fn block_name(self_: &DisplayInstruction<'_>, block_id: BasicBlockId) -> Name {
            self_
                .block_name_fn
                .as_ref()
                .and_then(|f| f(block_id))
                .unwrap_or_else(|| {
                    self_
                        .data
                        .block_ref(block_id)
                        .name
                        .to_owned()
                        .unwrap_or(Name::Number(block_id.index()))
                })
        }

        fn value_string(self_: &DisplayInstruction<'_>, val_id: ValueId) -> String {
            format!(
                "{}",
                self_
                    .data
                    .value_ref(val_id)
                    .display(&self_.data, &self_.types)
                    .display_type(false)
                    .display_as_operand(true)
                    .set_name_fn(Box::new(|v| match v {
                        Value::Instruction(id) => self_.inst_name_fn.as_ref().and_then(|f| f(*id)),
                        _ => None,
                    }))
            )
        }

        let dest = self
            .inst_name_fn
            .as_ref()
            .and_then(|f| f(self.inst.id.unwrap()))
            .unwrap_or_else(|| {
                self.inst
                    .dest
                    .to_owned()
                    .unwrap_or(Name::Number(self.inst.id.unwrap().index()))
            });

        match &self.inst.operand {
            Operand::Alloca(Alloca {
                tys,
                num_elements,
                align,
            }) => {
                write!(
                    f,
                    "%{dest:?} = alloca {}, {} {}{}",
                    self.types.to_string(tys[0]),
                    self.types.to_string(tys[1]),
                    num_elements.to_string(&self.types),
                    if *align > 0 {
                        format!(", align {}", align)
                    } else {
                        "".to_string()
                    }
                )
            }
            Operand::Phi(Phi { ty, args, blocks }) => {
                write!(
                    f,
                    "%{dest:?} = phi {} {}",
                    self.types.to_string(*ty),
                    args.iter()
                        .zip(blocks.iter())
                        .fold("".to_string(), |acc, (arg, &block)| {
                            format!(
                                "{}[{}, %{:?}], ",
                                acc,
                                value_string(&self, *arg),
                                block_name(&self, block)
                            )
                        })
                        .trim_end_matches(", ")
                )
            }
            Operand::Load(Load { tys, addr, align }) => {
                write!(
                    f,
                    "%{dest:?} = load {}, {} {}{}",
                    self.types.to_string(tys[0]),
                    self.types.to_string(tys[1]),
                    value_string(self, *addr),
                    if *align == 0 {
                        "".to_string()
                    } else {
                        format!(", align {}", align)
                    }
                )
            }
            Operand::Store(Store { tys, args, align }) => {
                write!(
                    f,
                    "store {} {}, {} {}{}",
                    self.types.to_string(tys[0]),
                    value_string(self, args[0]),
                    self.types.to_string(tys[1]),
                    value_string(self, args[1]),
                    if *align == 0 {
                        "".to_string()
                    } else {
                        format!(", align {}", align)
                    }
                )
            }
            Operand::InsertValue(InsertValue { tys, args }) => {
                write!(
                    f,
                    "%{dest:?} = insertvalue {} {}, {} {}, {}",
                    self.types.to_string(tys[0]),
                    value_string(self, args[0]),
                    self.types.to_string(tys[1]),
                    value_string(self, args[1]),
                    args[2..]
                        .iter()
                        .fold("".to_string(), |acc, &arg| {
                            format!("{}{}, ", acc, value_string(self, arg))
                        })
                        .trim_end_matches(", ")
                )
            }
            Operand::ExtractValue(ExtractValue { ty, args }) => {
                write!(
                    f,
                    "%{dest:?} = extractvalue {} {}, {}",
                    self.types.to_string(*ty),
                    value_string(self, args[0]),
                    args[1..]
                        .iter()
                        .fold("".to_string(), |acc, &arg| {
                            format!("{}{}, ", acc, value_string(self, arg))
                        })
                        .trim_end_matches(", ")
                )
            }
            Operand::IntBinary(IntBinary {
                ty,
                nuw,
                nsw,
                exact,
                args,
            }) => {
                write!(
                    f,
                    "%{dest:?} = {:?}{}{}{} {} {}, {}",
                    self.inst.opcode,
                    if *nuw { " nuw" } else { "" },
                    if *nsw { " nsw" } else { "" },
                    if *exact { " exact" } else { "" },
                    self.types.to_string(*ty),
                    value_string(self, args[0]),
                    value_string(self, args[1]),
                )
            }
            Operand::ICmp(ICmp { ty, args, cond }) => {
                write!(
                    f,
                    "%{dest:?} = icmp {:?} {} {}, {}",
                    cond,
                    self.types.to_string(*ty),
                    value_string(self, args[0]),
                    value_string(self, args[1])
                )
            }
            Operand::Cast(Cast { tys, arg }) => {
                write!(
                    f,
                    "%{dest:?} = {:?} {} {} to {}",
                    self.inst.opcode,
                    self.types.to_string(tys[0]),
                    value_string(self, *arg),
                    self.types.to_string(tys[1]),
                )
            }
            _ => todo!(),
        }
    }
}
