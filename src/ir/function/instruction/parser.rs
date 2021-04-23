use super::{ICmpCond, Instruction, InstructionId, Opcode, Operand};
use crate::ir::{
    function::{param_attrs::parser::parse_param_attrs, parser::ParserContext},
    module::attributes::parser::parse_attributes,
};
use crate::ir::{module::name, types, util::spaces, value};
use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::{char, digit1},
    combinator::{map, opt},
    error::VerboseError,
    sequence::{preceded, tuple},
    Err::Error,
    IResult,
};

pub fn parse_alloca<'a, 'b>(
    source: &'a str,
    ctx: &mut ParserContext<'b>,
) -> IResult<&'a str, Instruction, VerboseError<&'a str>> {
    let (source, _) = preceded(spaces, tag("alloca"))(source)?;
    let (source, ty) = types::parse(source, ctx.types)?;
    let (source, align) = opt(preceded(
        spaces,
        preceded(
            char(','),
            preceded(spaces, preceded(tag("align"), preceded(spaces, digit1))),
        ),
    ))(source)?;
    // TODO: Implement parser for num_elements
    let num_elements = value::ConstantData::Int(value::ConstantInt::Int32(1));
    let inst = Opcode::Alloca
        .with_block(ctx.cur_block)
        .with_operand(Operand::Alloca {
            tys: [ty, ctx.types.base().i32()],
            num_elements,
            align: align.map_or(0, |align| align.parse::<u32>().unwrap_or(0)),
        });
    Ok((source, inst))
}

pub fn parse_phi<'a, 'b>(
    source: &'a str,
    ctx: &mut ParserContext<'b>,
) -> IResult<&'a str, Instruction, VerboseError<&'a str>> {
    let (source, _) = preceded(spaces, tag("phi"))(source)?;
    let (mut source, ty) = types::parse(source, ctx.types)?;
    let mut args = vec![];
    let mut blocks = vec![];
    loop {
        let (source_, _) = preceded(spaces, char('['))(source)?;
        let (source_, arg) = value::parse(source_, ctx, ty)?;
        args.push(arg);
        let (source_, _) = preceded(spaces, char(','))(source_)?;
        let (source_, name) = preceded(spaces, preceded(char('%'), name::parse))(source_)?;
        let block = ctx.get_or_create_named_block(name);
        blocks.push(block);
        let (source_, _) = preceded(spaces, char(']'))(source_)?;
        if let Ok((source_, _)) = preceded(spaces, char(','))(source_) {
            source = source_;
            continue;
        }
        let inst = Opcode::Phi
            .with_block(ctx.cur_block)
            .with_operand(Operand::Phi { ty, args, blocks });
        return Ok((source_, inst));
    }
}

pub fn parse_load<'a, 'b>(
    source: &'a str,
    ctx: &mut ParserContext<'b>,
) -> IResult<&'a str, Instruction, VerboseError<&'a str>> {
    let (source, _) = preceded(spaces, tag("load"))(source)?;
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
    let inst = Opcode::Load
        .with_block(ctx.cur_block)
        .with_operand(Operand::Load {
            tys: [ty, addr_ty],
            addr,
            align: align.map_or(0, |align| align.parse::<u32>().unwrap_or(0)),
        });
    Ok((source, inst))
}

pub fn parse_store<'a, 'b>(
    source: &'a str,
    ctx: &mut ParserContext<'b>,
) -> IResult<&'a str, Instruction, VerboseError<&'a str>> {
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
        Opcode::Store
            .with_block(ctx.cur_block)
            .with_operand(Operand::Store {
                tys: [src_ty, dst_ty],
                args: [src, dst],
                align: align.map_or(0, |align| align.parse::<u32>().unwrap_or(0)),
            }),
    ))
}

pub fn parse_add_sub_mul<'a, 'b>(
    source: &'a str,
    ctx: &mut ParserContext<'b>,
) -> IResult<&'a str, Instruction, VerboseError<&'a str>> {
    let (source, opcode) = preceded(
        spaces,
        alt((
            map(tag("add"), |_| Opcode::Add),
            map(tag("sub"), |_| Opcode::Sub),
            map(tag("mul"), |_| Opcode::Mul),
        )),
    )(source)?;
    let (source, nuw) = opt(preceded(spaces, tag("nuw")))(source)?;
    let (source, nsw) = opt(preceded(spaces, tag("nsw")))(source)?;
    let (source, ty) = types::parse(source, ctx.types)?;
    let (source, lhs) = value::parse(source, ctx, ty)?;
    let (source, _) = preceded(spaces, char(','))(source)?;
    let (source, rhs) = value::parse(source, ctx, ty)?;
    let inst = opcode
        .with_block(ctx.cur_block)
        .with_operand(Operand::IntBinary {
            ty,
            args: [lhs, rhs],
            nuw: nuw.map_or(false, |_| true),
            nsw: nsw.map_or(false, |_| true),
        });
    Ok((source, inst))
}

pub fn parse_icmp<'a, 'b>(
    source: &'a str,
    ctx: &mut ParserContext<'b>,
) -> IResult<&'a str, Instruction, VerboseError<&'a str>> {
    pub fn icmp_cond<'a, 'b>(source: &'a str) -> IResult<&'a str, ICmpCond, VerboseError<&'a str>> {
        alt((
            map(tag("eq"), |_| ICmpCond::Eq),
            map(tag("ne"), |_| ICmpCond::Ne),
            map(tag("ugt"), |_| ICmpCond::Ugt),
            map(tag("uge"), |_| ICmpCond::Uge),
            map(tag("ult"), |_| ICmpCond::Ult),
            map(tag("ule"), |_| ICmpCond::Ule),
            map(tag("sgt"), |_| ICmpCond::Sgt),
            map(tag("sge"), |_| ICmpCond::Sge),
            map(tag("slt"), |_| ICmpCond::Slt),
            map(tag("sle"), |_| ICmpCond::Sle),
        ))(source)
    }

    let (source, _) = preceded(spaces, tag("icmp"))(source)?;
    let (source, cond) = preceded(spaces, icmp_cond)(source)?;
    let (source, ty) = types::parse(source, ctx.types)?;
    let (source, lhs) = value::parse(source, ctx, ty)?;
    let (source, _) = preceded(spaces, char(','))(source)?;
    let (source, rhs) = value::parse(source, ctx, ty)?;
    let inst = Opcode::ICmp
        .with_block(ctx.cur_block)
        .with_operand(Operand::ICmp {
            ty,
            args: [lhs, rhs],
            cond,
        });
    Ok((source, inst))
}

pub fn parse_cast<'a, 'b>(
    source: &'a str,
    ctx: &mut ParserContext<'b>,
) -> IResult<&'a str, Instruction, VerboseError<&'a str>> {
    let (source, opcode) = preceded(
        spaces,
        alt((
            map(tag("sext"), |_| Opcode::Sext),
            map(tag("zext"), |_| Opcode::Zext),
            map(tag("bitcast"), |_| Opcode::Bitcast),
        )),
    )(source)?;
    let (source, from) = types::parse(source, ctx.types)?;
    let (source, arg) = value::parse(source, ctx, from)?;
    let (source, _) = preceded(spaces, tag("to"))(source)?;
    let (source, to) = types::parse(source, ctx.types)?;
    let inst = opcode
        .with_block(ctx.cur_block)
        .with_operand(Operand::Cast {
            tys: [from, to],
            arg,
        });
    Ok((source, inst))
}

pub fn parse_call_args<'a, 'b>(
    source: &'a str,
    ctx: &mut ParserContext<'b>,
) -> IResult<&'a str, Vec<(types::TypeId, value::ValueId)>, VerboseError<&'a str>> {
    let (mut source, _) = preceded(spaces, char('('))(source)?;

    if let Ok((source, _)) = preceded(spaces, char(')'))(source) {
        return Ok((source, vec![]));
    }

    let mut args = vec![];
    loop {
        let (source_, ty) = types::parse(source, ctx.types)?;
        let (source_, arg) = value::parse(source_, ctx, ty)?;
        args.push((ty, arg));
        if let Ok((source_, _)) = preceded(spaces, char(','))(source_) {
            source = source_;
            continue;
        }
        if let Ok((source, _)) = preceded(spaces, char(')'))(source_) {
            return Ok((source, args));
        }
    }
}

pub fn parse_getelementptr<'a, 'b>(
    source: &'a str,
    ctx: &mut ParserContext<'b>,
) -> IResult<&'a str, Instruction, VerboseError<&'a str>> {
    let (source, _) = preceded(spaces, tag("getelementptr"))(source)?;
    let (source, inbounds) = opt(preceded(spaces, tag("inbounds")))(source)?;
    let (source, ty) = types::parse(source, ctx.types)?;
    let (mut source, _) = preceded(spaces, char(','))(source)?;
    let mut args = vec![];
    let mut tys = vec![ty];
    loop {
        let (source_, ty) = types::parse(source, ctx.types)?;
        let (source_, arg) = value::parse(source_, ctx, ty)?;
        tys.push(ty);
        args.push(arg);
        if let Ok((source_, _)) = preceded(spaces, char(','))(source_) {
            source = source_;
            continue;
        }
        let inst = Opcode::GetElementPtr
            .with_block(ctx.cur_block)
            .with_operand(Operand::GetElementPtr {
                inbounds: inbounds.is_some(),
                tys,
                args,
            });
        return Ok((source_, inst));
    }
}

pub fn parse_call<'a, 'b>(
    source: &'a str,
    ctx: &mut ParserContext<'b>,
) -> IResult<&'a str, Instruction, VerboseError<&'a str>> {
    let (source, _) = preceded(spaces, tag("call"))(source)?;
    let (source, ret_attrs) = parse_param_attrs(source)?;
    let (source, ty) = types::parse(source, ctx.types)?;
    let (source, callee) = value::parse(source, ctx, ty)?;
    let (source, args_) = parse_call_args(source, ctx)?;
    let (source, func_attrs) = parse_attributes(source)?;
    let mut tys = vec![ty];
    let mut args = vec![callee];
    for (t, a) in args_ {
        tys.push(t);
        args.push(a);
    }
    let inst = Opcode::Call
        .with_block(ctx.cur_block)
        .with_operand(Operand::Call {
            tys,
            args,
            ret_attrs,
            func_attrs,
        });
    Ok((source, inst))
}

pub fn parse_br<'a, 'b>(
    source: &'a str,
    ctx: &mut ParserContext<'b>,
) -> IResult<&'a str, Instruction, VerboseError<&'a str>> {
    let (source, _) = preceded(spaces, tag("br"))(source)?;
    if let Ok((source, _)) = preceded(spaces, tag("label"))(source) {
        let (source, label) = preceded(spaces, preceded(char('%'), name::parse))(source)?;
        let block = ctx.get_or_create_named_block(label);
        let inst = Opcode::Br
            .with_block(ctx.cur_block)
            .with_operand(Operand::Br { block });
        Ok((source, inst))
    } else {
        let (source, ty) = types::parse(source, ctx.types)?;
        assert_eq!(*ctx.types.get(ty), types::Type::Int(1));
        let (source, arg) = value::parse(source, ctx, ty)?;
        let (source, _) = preceded(spaces, char(','))(source)?;
        let (source, iftrue) = preceded(
            spaces,
            preceded(
                tag("label"),
                preceded(spaces, preceded(char('%'), name::parse)),
            ),
        )(source)?;
        let (source, _) = preceded(spaces, char(','))(source)?;
        let (source, iffalse) = preceded(
            spaces,
            preceded(
                tag("label"),
                preceded(spaces, preceded(char('%'), name::parse)),
            ),
        )(source)?;
        let iftrue = ctx.get_or_create_named_block(iftrue);
        let iffalse = ctx.get_or_create_named_block(iffalse);
        let inst = Opcode::CondBr
            .with_block(ctx.cur_block)
            .with_operand(Operand::CondBr {
                arg,
                blocks: [iftrue, iffalse],
            });
        Ok((source, inst))
    }
}

pub fn parse_ret<'a, 'b>(
    source: &'a str,
    ctx: &mut ParserContext<'b>,
) -> IResult<&'a str, Instruction, VerboseError<&'a str>> {
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
        Opcode::Ret
            .with_block(ctx.cur_block)
            .with_operand(Operand::Ret { val, ty }),
    ))
}

/// Only parses `source` as Instruction. Doesn't append instruction to block.
pub fn parse<'a, 'b>(
    source: &'a str,
    ctx: &mut ParserContext<'b>,
) -> IResult<&'a str, InstructionId, VerboseError<&'a str>> {
    let (source, name) = opt(tuple((spaces, char('%'), name::parse, spaces, char('='))))(source)?;
    let name = name.map_or(None, |(_, _, name, _, _)| Some(name));
    for f in [
        parse_alloca,
        parse_phi,
        parse_load,
        parse_store,
        parse_add_sub_mul,
        parse_icmp,
        parse_cast,
        parse_getelementptr,
        parse_call,
        parse_br,
        parse_ret,
    ]
    .iter()
    {
        if let Ok((source, inst)) = f(source, ctx) {
            if let Some(name) = name {
                if let Some(inner) = ctx.name_to_value.get(&name) {
                    if let value::Value::Instruction(id) = ctx.data.values[*inner] {
                        ctx.data.replace_inst(id, inst.with_dest(name));
                        return Ok((source, id));
                    }
                }

                let id = ctx.data.create_inst(inst.with_dest(name.clone()));
                ctx.name_to_value
                    .insert(name, ctx.data.create_value(value::Value::Instruction(id)));
                return Ok((source, id));
            }

            return Ok((source, ctx.data.create_inst(inst)));
        }
    }
    Err(Error(VerboseError { errors: vec![] }))
}
