use super::{
    super::{function::parser::ParserContext, types, util::spaces, value},
    InstructionId, Opcode, Operand,
};
use nom::{
    bytes::complete::tag,
    // combinator::map,
    error::VerboseError,
    sequence::preceded,
    IResult,
};

pub fn parse_ret<'a, 'b>(
    source: &'a str,
    ctx: &mut ParserContext<'b>,
) -> IResult<&'a str, InstructionId, VerboseError<&'a str>> {
    let (source, _) = preceded(spaces, preceded(tag("ret"), spaces))(source)?;
    let is_void: IResult<&'a str, &'a str, VerboseError<&'a str>> = tag("void")(source);

    if let Ok((source, _)) = is_void {
        return Ok((
            source,
            ctx.data.create_inst(
                Opcode::Ret
                    .with_block(ctx.cur_block)
                    .with_operand(Operand::Ret { val: None }),
            ),
        ));
    }

    let (source, ty) = types::parse(source, ctx.types)?;
    let (source, val) = value::parse(source, ctx, ty)?;

    Ok((
        source,
        ctx.data.create_inst(
            Opcode::Ret
                .with_block(ctx.cur_block)
                .with_operand(Operand::Ret { val: Some(val) }),
        ),
    ))
}

/// Only parses `source` as Instruction. Doesn't append instruction to block.
pub fn parse<'a, 'b>(
    source: &'a str,
    ctx: &mut ParserContext<'b>,
    // data: &mut Data,
    // layout: &mut Layout,
    // types: &Types,
) -> IResult<&'a str, InstructionId, VerboseError<&'a str>> {
    // alt((inst1, inst2, ...))
    parse_ret(source, ctx)
}
