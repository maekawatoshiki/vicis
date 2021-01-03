use id_arena::Id;

pub type InstructionId<Data> = Id<Instruction<Data>>;

#[derive(Debug)]
pub struct Instruction<Data> {
    pub id: Option<InstructionId<Data>>,
    pub data: Data,
}
