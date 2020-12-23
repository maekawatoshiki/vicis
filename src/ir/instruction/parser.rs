use super::{
    super::{function::parser::ParserContext, module::name, types, util::spaces, value},
    InstructionId, Opcode, Operand,
};
use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::{char, digit1},
    combinator::{map, opt},
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
    let (source, align) = opt(preceded(
        spaces,
        preceded(
            char(','),
            preceded(spaces, preceded(tag("align"), preceded(spaces, digit1))),
        ),
    ))(source)?;
    // TODO: Implement parser for num_elements
    let num_elements = ctx
        .data
        .create_value(value::Value::Constant(value::ConstantData::Int(
            value::ConstantInt::Int32(1),
        )));
    let inst_id = ctx.data.create_inst(
        Opcode::Alloca
            .with_block(ctx.cur_block)
            .with_dest(name.clone())
            .with_operand(Operand::Alloca {
                tys: [ty, ctx.types.base().i32()],
                num_elements,
                align: align.map_or(0, |align| align.parse::<u32>().unwrap_or(0)),
            }),
    );
    let inst_val_id = ctx.data.create_value(value::Value::Instruction(inst_id));
    ctx.name_to_value.insert(name, inst_val_id);
    Ok((source, inst_id))
}

pub fn parse_load<'a, 'b>(
    source: &'a str,
    ctx: &mut ParserContext<'b>,
) -> IResult<&'a str, InstructionId, VerboseError<&'a str>> {
    let (source, name) = preceded(spaces, preceded(char('%'), name::parse))(source)?;
    let (source, _) = preceded(
        spaces,
        preceded(char('='), preceded(spaces, preceded(tag("load"), spaces))),
    )(source)?;
    let (source, ty) = types::parse(source, ctx.types)?;
    let (source, _) = preceded(spaces, char(','))(source)?;
    let (source, addr_ty) = types::parse(source, ctx.types)?;
    let (source, addr) = value::parse(source, ctx, addr_ty)?;
    let (source, align) = opt(preceded(
        spaces,
        preceded(
            char(','),
            preceded(spaces, preceded(tag("align"), preceded(spaces, digit1))),
        ),
    ))(source)?;
    let inst_id = ctx.data.create_inst(
        Opcode::Load
            .with_block(ctx.cur_block)
            .with_dest(name.clone())
            .with_operand(Operand::Load {
                tys: [ty, addr_ty],
                addr,
                align: align.map_or(0, |align| align.parse::<u32>().unwrap_or(0)),
            }),
    );
    let inst_val_id = ctx.data.create_value(value::Value::Instruction(inst_id));
    ctx.name_to_value.insert(name, inst_val_id);
    Ok((source, inst_id))
}

pub fn parse_store<'a, 'b>(
    source: &'a str,
    ctx: &mut ParserContext<'b>,
) -> IResult<&'a str, InstructionId, VerboseError<&'a str>> {
    let (source, _) = preceded(spaces, preceded(tag("store"), spaces))(source)?;
    let (source, src_ty) = types::parse(source, ctx.types)?;
    let (source, src) = value::parse(source, ctx, src_ty)?;
    let (source, _) = preceded(spaces, char(','))(source)?;
    let (source, dst_ty) = types::parse(source, ctx.types)?;
    let (source, dst) = value::parse(source, ctx, dst_ty)?;
    let (source, align) = opt(preceded(
        spaces,
        preceded(
            char(','),
            preceded(spaces, preceded(tag("align"), preceded(spaces, digit1))),
        ),
    ))(source)?;
    Ok((
        source,
        ctx.data
            .create_inst(
                Opcode::Store
                    .with_block(ctx.cur_block)
                    .with_operand(Operand::Store {
                        tys: [src_ty, dst_ty],
                        args: [src, dst],
                        align: align.map_or(0, |align| align.parse::<u32>().unwrap_or(0)),
                    }),
            ),
    ))
}

pub fn parse_add_sub_mul<'a, 'b>(
    source: &'a str,
    ctx: &mut ParserContext<'b>,
) -> IResult<&'a str, InstructionId, VerboseError<&'a str>> {
    let (source, name) = preceded(spaces, preceded(char('%'), name::parse))(source)?;
    let (source, opcode) = preceded(
        spaces,
        preceded(
            char('='),
            preceded(
                spaces,
                alt((
                    map(tag("add"), |_| Opcode::Add),
                    map(tag("sub"), |_| Opcode::Sub),
                    map(tag("mul"), |_| Opcode::Mul),
                )),
            ),
        ),
    )(source)?;
    let (source, nuw) = opt(preceded(spaces, tag("nuw")))(source)?;
    let (source, nsw) = opt(preceded(spaces, tag("nsw")))(source)?;
    let (source, ty) = types::parse(source, ctx.types)?;
    let (source, lhs) = value::parse(source, ctx, ty)?;
    let (source, _) = preceded(spaces, char(','))(source)?;
    let (source, rhs) = value::parse(source, ctx, ty)?;
    let inst_id = ctx.data.create_inst(
        opcode
            .with_block(ctx.cur_block)
            .with_dest(name.clone())
            .with_operand(Operand::IntBinary {
                ty,
                args: [lhs, rhs],
                nuw: nuw.map_or(false, |_| true),
                nsw: nsw.map_or(false, |_| true),
            }),
    );
    let inst_val_id = ctx.data.create_value(value::Value::Instruction(inst_id));
    ctx.name_to_value.insert(name, inst_val_id);
    Ok((source, inst_id))
}

pub fn parse_ret<'a, 'b>(
    source: &'a str,
    ctx: &mut ParserContext<'b>,
) -> IResult<&'a str, InstructionId, VerboseError<&'a str>> {
    let (source, _) = preceded(spaces, preceded(tag("ret"), spaces))(source)?;
    let (source, ty) = types::parse(source, ctx.types)?;
    let (source, val) = if *ctx.types.get(ty) == types::Type::Void {
        (source, None)
    } else {
        let (source, val) = value::parse(source, ctx, ty)?;
        (source, Some(val))
    };
    Ok((
        source,
        ctx.data.create_inst(
            Opcode::Ret
                .with_block(ctx.cur_block)
                .with_operand(Operand::Ret { val, ty }),
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

    if let Ok((source, id)) = parse_load(source, ctx) {
        return Ok((source, id));
    }

    if let Ok((source, id)) = parse_store(source, ctx) {
        return Ok((source, id));
    }

    if let Ok((source, id)) = parse_add_sub_mul(source, ctx) {
        return Ok((source, id));
    }

    parse_ret(source, ctx)
}
