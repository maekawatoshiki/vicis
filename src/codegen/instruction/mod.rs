use id_arena::Id;

pub type InstructionId<Data> = Id<Instruction<Data>>;

pub struct Instruction<Data> {
    pub data: Data,
}
