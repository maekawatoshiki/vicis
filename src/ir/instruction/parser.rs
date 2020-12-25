use super::{
    super::{function::parser::ParserContext, module::name, types, util::spaces, value},
    ICmpCond, InstructionId, Opcode, Operand,
};
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

pub fn parse_icmp<'a, 'b>(
    source: &'a str,
    ctx: &mut ParserContext<'b>,
) -> IResult<&'a str, InstructionId, VerboseError<&'a str>> {
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

    let (source, name) = preceded(spaces, preceded(char('%'), name::parse))(source)?;
    let (source, _) = preceded(spaces, preceded(char('='), preceded(spaces, tag("icmp"))))(source)?;
    let (source, cond) = preceded(spaces, icmp_cond)(source)?;
    let (source, ty) = types::parse(source, ctx.types)?;
    let (source, lhs) = value::parse(source, ctx, ty)?;
    let (source, _) = preceded(spaces, char(','))(source)?;
    let (source, rhs) = value::parse(source, ctx, ty)?;
    let inst_id = ctx.data.create_inst(
        Opcode::ICmp
            .with_block(ctx.cur_block)
            .with_dest(name.clone())
            .with_operand(Operand::ICmp {
                ty,
                args: [lhs, rhs],
                cond,
            }),
    );
    let inst_val_id = ctx.data.create_value(value::Value::Instruction(inst_id));
    ctx.name_to_value.insert(name, inst_val_id);
    Ok((source, inst_id))
}

pub fn parse_zext<'a, 'b>(
    source: &'a str,
    ctx: &mut ParserContext<'b>,
) -> IResult<&'a str, InstructionId, VerboseError<&'a str>> {
    let (source, name) = preceded(spaces, preceded(char('%'), name::parse))(source)?;
    let (source, _) = preceded(spaces, preceded(char('='), preceded(spaces, tag("zext"))))(source)?;
    let (source, from) = types::parse(source, ctx.types)?;
    let (source, arg) = value::parse(source, ctx, from)?;
    let (source, _) = preceded(spaces, tag("to"))(source)?;
    let (source, to) = types::parse(source, ctx.types)?;
    let inst_id = ctx.data.create_inst(
        Opcode::Zext
            .with_block(ctx.cur_block)
            .with_dest(name.clone())
            .with_operand(Operand::Cast {
                tys: [from, to],
                arg,
            }),
    );
    let inst_val_id = ctx.data.create_value(value::Value::Instruction(inst_id));
    ctx.name_to_value.insert(name, inst_val_id);
    Ok((source, inst_id))
}

pub fn parse_call_args<'a, 'b>(
    source: &'a str,
    ctx: &mut ParserContext<'b>,
) -> IResult<&'a str, Vec<(types::TypeId, value::ValueId)>, VerboseError<&'a str>> {
    let (mut source, _) = preceded(spaces, char('('))(source)?;
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

pub fn parse_call<'a, 'b>(
    source: &'a str,
    ctx: &mut ParserContext<'b>,
) -> IResult<&'a str, InstructionId, VerboseError<&'a str>> {
    let (source, name) = preceded(spaces, preceded(char('%'), name::parse))(source)?;
    let (source, _) = preceded(spaces, preceded(char('='), preceded(spaces, tag("call"))))(source)?;
    let (source, ty) = types::parse(source, ctx.types)?;
    let (source, callee) = value::parse(source, ctx, ty)?;
    let (source, args_) = parse_call_args(source, ctx)?;
    let mut tys = vec![ty];
    let mut args = vec![callee];
    for (t, a) in args_ {
        tys.push(t);
        args.push(a);
    }
    let inst_id = ctx.data.create_inst(
        Opcode::Call
            .with_block(ctx.cur_block)
            .with_dest(name.clone())
            .with_operand(Operand::Call { tys, args }),
    );
    let inst_val_id = ctx.data.create_value(value::Value::Instruction(inst_id));
    ctx.name_to_value.insert(name, inst_val_id);
    Ok((source, inst_id))
}

pub fn parse_br<'a, 'b>(
    source: &'a str,
    ctx: &mut ParserContext<'b>,
) -> IResult<&'a str, InstructionId, VerboseError<&'a str>> {
    let (source, _) = preceded(spaces, tag("br"))(source)?;
    if let Ok((source, _)) = preceded(spaces, tag("label"))(source) {
        let (source, label) = preceded(spaces, preceded(char('%'), name::parse))(source)?;
        let block = ctx.get_or_create_named_block(label);
        let inst_id = ctx.data.create_inst(
            Opcode::Br
                .with_block(ctx.cur_block)
                .with_operand(Operand::Br { block }),
        );
        Ok((source, inst_id))
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
        let inst_id =
            ctx.data
                .create_inst(Opcode::CondBr.with_block(ctx.cur_block).with_operand(
                    Operand::CondBr {
                        arg,
                        blocks: [iftrue, iffalse],
                    },
                ));
        Ok((source, inst_id))
    }
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
    let mut res = Err(Error(VerboseError { errors: vec![] }));
    for f in [
        parse_alloca,
        parse_load,
        parse_store,
        parse_add_sub_mul,
        parse_icmp,
        parse_zext,
        parse_call,
        parse_br,
        parse_ret,
    ]
    .iter()
    {
        res = f(source, ctx);
        if let Ok((source, id)) = res {
            return Ok((source, id));
        }
    }
    res
}
