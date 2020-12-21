use super::{
    super::{function::parser::ParserContext, module::name, types, util::spaces, value},
    InstructionId, Opcode, Operand,
};
use nom::{
    bytes::complete::tag,
    character::complete::{char, digit1},
    combinator::opt,
    error::VerboseError,
    sequence::{preceded, tuple},
    IResult,
};

pub fn parse_alloca<'a, 'b>(
    source: &'a str,
    ctx: &mut ParserContext<'b>,
) -> IResult<&'a str, InstructionId, VerboseError<&'a str>> {
    let (source, name) = preceded(spaces, preceded(char('%'), name::parse))(source)?;
    let (source, _) = tuple((spaces, char('='), spaces, tag("alloca"), spaces))(source)?;
    let (source, ty) = types::parse(source, ctx.types)?;
    // let (source, num_elements) =
    let (source, align) = opt(preceded(
        spaces,
        preceded(
            char(','),
            preceded(spaces, preceded(tag("align"), preceded(spaces, digit1))),
        ),
    ))(source)?;
    Ok((
        source,
        ctx.data.create_inst(
            Opcode::Alloca
                .with_block(ctx.cur_block)
                .with_dest(name)
                .with_operand(Operand::Alloca {
                    ty,
                    num_elements: value::ConstantData::Int(value::ConstantInt::Int32(1)),
                    align: align.map_or(0, |align| align.parse::<u32>().unwrap_or(0)),
                }),
        ),
    ))
}

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
) -> IResult<&'a str, InstructionId, VerboseError<&'a str>> {
    if let Ok((source, id)) = parse_alloca(source, ctx) {
        return Ok((source, id));
    }

    parse_ret(source, ctx)
}
