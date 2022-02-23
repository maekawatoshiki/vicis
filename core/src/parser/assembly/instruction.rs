use super::attributes::parse_attributes;
use super::function::ParserContext;
use super::name::identifier;
use super::param_attrs::parse_param_attrs;
use crate::ir::function::instruction::{
    Alloca, Br, Call, Cast, CondBr, GetElementPtr, ICmp, ICmpCond, Instruction, InstructionId,
    IntBinary, Invoke, LandingPad, Load, Opcode, Operand, Phi, Resume, Ret, Store, Switch,
};
use crate::ir::{
    function::{
        instruction::{ExtractValue, InsertValue},
        param_attrs::ParameterAttribute,
    },
    module::metadata::Metadata,
    types::I32,
    util::string_literal,
};
use crate::ir::{types, util::spaces, value};
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
use rustc_hash::FxHashMap;

pub fn parse_alloca<'a, 'b>(
    source: &'a str,
    ctx: &mut ParserContext<'b>,
) -> IResult<&'a str, Instruction, VerboseError<&'a str>> {
    let (source, _) = preceded(spaces, tag("alloca"))(source)?;
    let (source, ty) = super::types::parse(ctx.types)(source)?;
    let (source, align) = opt(preceded(
        spaces,
        preceded(
            char(','),
            preceded(spaces, preceded(tag("align"), preceded(spaces, digit1))),
        ),
    ))(source)?;
    // TODO: Implement parser for num_elements
    let num_elements = value::ConstantValue::Int(value::ConstantInt::Int32(1));
    let inst = Opcode::Alloca
        .with_block(ctx.cur_block)
        .with_operand(Operand::Alloca(Alloca {
            tys: [ty, I32],
            num_elements,
            align: align.map_or(0, |align| align.parse::<u32>().unwrap_or(0)),
        }))
        .with_ty(ctx.types.base_mut().pointer(ty));
    Ok((source, inst))
}

pub fn parse_phi<'a, 'b>(
    source: &'a str,
    ctx: &mut ParserContext<'b>,
) -> IResult<&'a str, Instruction, VerboseError<&'a str>> {
    let (source, _) = preceded(spaces, tag("phi"))(source)?;
    let (mut source, ty) = super::types::parse(ctx.types)(source)?;
    let mut args = vec![];
    let mut blocks = vec![];
    loop {
        let (source_, _) = preceded(spaces, char('['))(source)?;
        let (source_, arg) = super::value::parse(source_, ctx, ty)?;
        args.push(arg);
        let (source_, _) = preceded(spaces, char(','))(source_)?;
        let (source_, name) = preceded(spaces, preceded(char('%'), super::name::parse))(source_)?;
        let block = ctx.get_or_create_named_block(name);
        blocks.push(block);
        let (source_, _) = preceded(spaces, char(']'))(source_)?;
        if let Ok((source_, _)) = preceded(spaces, char(','))(source_) {
            source = source_;
            continue;
        }
        let inst = Opcode::Phi
            .with_block(ctx.cur_block)
            .with_operand(Operand::Phi(Phi { ty, args, blocks }))
            .with_ty(ty);
        return Ok((source_, inst));
    }
}

pub fn parse_load<'a, 'b>(
    source: &'a str,
    ctx: &mut ParserContext<'b>,
) -> IResult<&'a str, Instruction, VerboseError<&'a str>> {
    let (source, _) = preceded(spaces, tag("load"))(source)?;
    let (source, ty) = super::types::parse(ctx.types)(source)?;
    let (source, _) = preceded(spaces, char(','))(source)?;
    let (source, addr_ty) = super::types::parse(ctx.types)(source)?;
    let (source, addr) = super::value::parse(source, ctx, addr_ty)?;
    let (source, align) = opt(preceded(
        spaces,
        preceded(
            char(','),
            preceded(spaces, preceded(tag("align"), preceded(spaces, digit1))),
        ),
    ))(source)?;
    let (source, _) = opt(parse_metadata("!nonnull"))(source)?; // TODO: FIXME: don't ignore !nonnull
    let (source, _) = opt(parse_metadata("!range"))(source)?; // TODO: FIXME: don't ignore !range
    let inst = Opcode::Load
        .with_block(ctx.cur_block)
        .with_operand(Operand::Load(Load {
            tys: [ty, addr_ty],
            addr,
            align: align.map_or(0, |align| align.parse::<u32>().unwrap_or(0)),
        }))
        .with_ty(ty);
    Ok((source, inst))
}

pub fn parse_store<'a, 'b>(
    source: &'a str,
    ctx: &mut ParserContext<'b>,
) -> IResult<&'a str, Instruction, VerboseError<&'a str>> {
    let (source, _) = preceded(spaces, preceded(tag("store"), spaces))(source)?;
    let (source, src_ty) = super::types::parse(ctx.types)(source)?;
    let (source, src) = super::value::parse(source, ctx, src_ty)?;
    let (source, _) = preceded(spaces, char(','))(source)?;
    let (source, dst_ty) = super::types::parse(ctx.types)(source)?;
    let (source, dst) = super::value::parse(source, ctx, dst_ty)?;
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
            .with_operand(Operand::Store(Store {
                tys: [src_ty, dst_ty],
                args: [src, dst],
                align: align.map_or(0, |align| align.parse::<u32>().unwrap_or(0)),
            })),
    ))
}

pub fn parse_insertvalue<'a, 'b>(
    source: &'a str,
    ctx: &mut ParserContext<'b>,
) -> IResult<&'a str, Instruction, VerboseError<&'a str>> {
    let (source, _) = preceded(spaces, tag("insertvalue"))(source)?;
    let (source, aggre_ty) = super::types::parse(ctx.types)(source)?;
    let (source, val) = super::value::parse(source, ctx, aggre_ty)?;
    let (source, _) = preceded(spaces, char(','))(source)?;
    let (source, ty) = super::types::parse(ctx.types)(source)?;
    let (source, elt) = super::value::parse(source, ctx, ty)?;
    let (source, _) = preceded(spaces, char(','))(source)?;
    let mut args = vec![val, elt];
    let (mut source, idx) = super::value::parse(source, ctx, I32)?;
    args.push(idx);
    loop {
        if let Ok((source_, _)) = preceded(spaces, char(','))(source) {
            let (source_, idx) = super::value::parse(source_, ctx, I32)?;
            args.push(idx);
            source = source_;
            continue;
        }
        return Ok((
            source,
            Opcode::InsertValue
                .with_block(ctx.cur_block)
                .with_operand(Operand::InsertValue(InsertValue {
                    tys: [aggre_ty, ty],
                    args,
                }))
                .with_ty(aggre_ty),
        ));
    }
}

pub fn parse_extractvalue<'a, 'b>(
    source: &'a str,
    ctx: &mut ParserContext<'b>,
) -> IResult<&'a str, Instruction, VerboseError<&'a str>> {
    let (source, _) = preceded(spaces, tag("extractvalue"))(source)?;
    let (source, aggre_ty) = super::types::parse(ctx.types)(source)?;
    let (source, val) = super::value::parse(source, ctx, aggre_ty)?;
    let (source, _) = preceded(spaces, char(','))(source)?;
    let mut args = vec![val];
    let (mut source, idx) = super::value::parse(source, ctx, I32)?;
    args.push(idx);
    loop {
        if let Ok((source_, _)) = preceded(spaces, char(','))(source) {
            let (source_, idx) = super::value::parse(source_, ctx, I32)?;
            args.push(idx);
            source = source_;
            continue;
        }
        return Ok((
            source,
            Opcode::ExtractValue
                .with_block(ctx.cur_block)
                .with_operand(Operand::ExtractValue(ExtractValue { ty: aggre_ty, args })),
        ));
    }
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
            map(tag("sdiv"), |_| Opcode::SDiv),
            map(tag("srem"), |_| Opcode::SRem),
            map(tag("and"), |_| Opcode::And),
            map(tag("or"), |_| Opcode::Or),
            map(tag("shl"), |_| Opcode::Shl),
            map(tag("ashr"), |_| Opcode::AShr),
            map(tag("lshr"), |_| Opcode::LShr),
        )),
    )(source)?;
    // TODO: `and` doesn't need nuw/nsw/exact keywords. We had better show error when they appear.
    let (source, nuw) = opt(preceded(spaces, tag("nuw")))(source)?;
    let (source, nsw) = opt(preceded(spaces, tag("nsw")))(source)?;
    let (source, exact) = opt(preceded(spaces, tag("exact")))(source)?;
    let (source, ty) = super::types::parse(ctx.types)(source)?;
    let (source, lhs) = super::value::parse(source, ctx, ty)?;
    let (source, _) = preceded(spaces, char(','))(source)?;
    let (source, rhs) = super::value::parse(source, ctx, ty)?;
    let inst = opcode
        .with_block(ctx.cur_block)
        .with_operand(Operand::IntBinary(IntBinary {
            ty,
            args: [lhs, rhs],
            nuw: nuw.map_or(false, |_| true),
            nsw: nsw.map_or(false, |_| true),
            exact: exact.map_or(false, |_| true),
        }));
    Ok((source, inst))
}

pub fn parse_icmp<'a, 'b>(
    source: &'a str,
    ctx: &mut ParserContext<'b>,
) -> IResult<&'a str, Instruction, VerboseError<&'a str>> {
    pub fn icmp_cond(source: &str) -> IResult<&str, ICmpCond, VerboseError<&str>> {
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
    let (source, ty) = super::types::parse(ctx.types)(source)?;
    let (source, lhs) = super::value::parse(source, ctx, ty)?;
    let (source, _) = preceded(spaces, char(','))(source)?;
    let (source, rhs) = super::value::parse(source, ctx, ty)?;
    let inst = Opcode::ICmp
        .with_block(ctx.cur_block)
        .with_operand(Operand::ICmp(ICmp {
            ty,
            args: [lhs, rhs],
            cond,
        }));
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
            map(tag("trunc"), |_| Opcode::Trunc),
            map(tag("inttoptr"), |_| Opcode::IntToPtr),
            map(tag("ptrtoint"), |_| Opcode::PtrToInt),
        )),
    )(source)?;
    let (source, from) = super::types::parse(ctx.types)(source)?;
    let (source, arg) = super::value::parse(source, ctx, from)?;
    let (source, _) = preceded(spaces, tag("to"))(source)?;
    let (source, to) = super::types::parse(ctx.types)(source)?;
    let inst = opcode
        .with_block(ctx.cur_block)
        .with_operand(Operand::Cast(Cast {
            tys: [from, to],
            arg,
        }));
    Ok((source, inst))
}

type CallArguments = (
    Vec<types::Type>,
    Vec<Vec<ParameterAttribute>>,
    Vec<value::ValueId>,
);

pub fn parse_call_args<'a, 'b>(
    source: &'a str,
    ctx: &mut ParserContext<'b>,
) -> IResult<&'a str, CallArguments, VerboseError<&'a str>> {
    let (mut source, _) = preceded(spaces, char('('))(source)?;

    if let Ok((source, _)) = preceded(spaces, char(')'))(source) {
        return Ok((source, (vec![], vec![], vec![])));
    }

    let mut arg_types = vec![];
    let mut arg_attr_lists = vec![];
    let mut arg_values = vec![];
    loop {
        let (source_, ty) = super::types::parse(ctx.types)(source)?;
        let (source_, attrs) = parse_param_attrs(source_, ctx.types)?;
        let (source_, arg) = super::value::parse(source_, ctx, ty)?;
        arg_types.push(ty);
        arg_attr_lists.push(attrs);
        arg_values.push(arg);

        if let Ok((source_, _)) = preceded(spaces, char(','))(source_) {
            source = source_;
            continue;
        }
        if let Ok((source, _)) = preceded(spaces, char(')'))(source_) {
            return Ok((source, (arg_types, arg_attr_lists, arg_values)));
        }
    }
}

pub fn parse_getelementptr<'a, 'b>(
    source: &'a str,
    ctx: &mut ParserContext<'b>,
) -> IResult<&'a str, Instruction, VerboseError<&'a str>> {
    let (source, _) = preceded(spaces, tag("getelementptr"))(source)?;
    let (source, inbounds) = opt(preceded(spaces, tag("inbounds")))(source)?;
    let (source, ty) = super::types::parse(ctx.types)(source)?;
    let (mut source, _) = preceded(spaces, char(','))(source)?;
    let mut args = vec![];
    let mut tys = vec![ty];
    loop {
        let (source_, ty) = super::types::parse(ctx.types)(source)?;
        let (source_, arg) = super::value::parse(source_, ctx, ty)?;
        tys.push(ty);
        args.push(arg);
        if let Ok((source_, _)) = preceded(spaces, char(','))(source_) {
            source = source_;
            continue;
        }
        let inst = Opcode::GetElementPtr
            .with_block(ctx.cur_block)
            .with_operand(Operand::GetElementPtr(GetElementPtr {
                inbounds: inbounds.is_some(),
                tys,
                args,
            }));
        return Ok((source_, inst));
    }
}

pub fn parse_call<'a, 'b>(
    source: &'a str,
    ctx: &mut ParserContext<'b>,
) -> IResult<&'a str, Instruction, VerboseError<&'a str>> {
    let (source, _) = preceded(spaces, tag("call"))(source)?;
    let (source, ret_attrs) = parse_param_attrs(source, ctx.types)?;
    let (source, ty) = super::types::parse(ctx.types)(source)?;
    let (source, callee) = parse_callee(source, ctx, ty)?;
    let (source, (mut tys, param_attrs, mut args)) = parse_call_args(source, ctx)?;
    let (source, func_attrs) = parse_attributes(source)?;
    let (source, _) = opt(parse_metadata("!srcloc"))(source)?; // TODO: FIXME: don't ignore !srcloc
    tys.insert(0, ty);
    args.insert(0, callee);
    let inst = Opcode::Call
        .with_block(ctx.cur_block)
        .with_operand(Operand::Call(Call {
            tys,
            args,
            param_attrs,
            ret_attrs,
            func_attrs,
        }));
    Ok((source, inst))
}

pub fn parse_callee<'a, 'b>(
    source: &'a str,
    ctx: &mut ParserContext<'b>,
    ty: types::Type,
) -> IResult<&'a str, value::ValueId, VerboseError<&'a str>> {
    if let Ok((source, asm)) = parse_call_asm(source) {
        return Ok((source, ctx.data.create_value(value::Value::InlineAsm(asm))));
    }

    let (source, callee) = super::value::parse(source, ctx, ty)?;
    Ok((source, callee))
}

pub fn parse_call_asm(source: &str) -> IResult<&str, value::InlineAsm, VerboseError<&str>> {
    let (source, _) = preceded(spaces, tag("asm"))(source)?;
    let (source, sideeffect) = opt(tuple((spaces, tag("sideeffect"))))(source)?;
    let (source, constraints) = preceded(spaces, string_literal)(source)?;
    let (source, _) = preceded(spaces, char(','))(source)?;
    let (source, body) = preceded(spaces, string_literal)(source)?;
    Ok((
        source,
        value::InlineAsm {
            constraints,
            body,
            sideeffect: sideeffect.is_some(),
        },
    ))
}

pub fn parse_invoke<'a, 'b>(
    source: &'a str,
    ctx: &mut ParserContext<'b>,
) -> IResult<&'a str, Instruction, VerboseError<&'a str>> {
    let (source, _) = preceded(spaces, tag("invoke"))(source)?;
    let (source, ret_attrs) = parse_param_attrs(source, ctx.types)?;
    let (source, ty) = super::types::parse(ctx.types)(source)?;
    let (source, callee) = super::value::parse(source, ctx, ty)?;
    let (source, (mut tys, param_attrs, mut args)) = parse_call_args(source, ctx)?;
    tys.insert(0, ty);
    args.insert(0, callee);
    let (source, func_attrs) = parse_attributes(source)?;
    let (source, (_, _, _, _, _, _, normal)) = tuple((
        spaces,
        tag("to"),
        spaces,
        tag("label"),
        spaces,
        char('%'),
        super::name::parse,
    ))(source)?;
    let (source, (_, _, _, _, _, _, exception)) = tuple((
        spaces,
        tag("unwind"),
        spaces,
        tag("label"),
        spaces,
        char('%'),
        super::name::parse,
    ))(source)?;
    let normal = ctx.get_or_create_named_block(normal);
    let exception = ctx.get_or_create_named_block(exception);
    let inst = Opcode::Invoke
        .with_block(ctx.cur_block)
        .with_operand(Operand::Invoke(Invoke {
            tys,
            args,
            param_attrs,
            ret_attrs,
            func_attrs,
            blocks: vec![normal, exception],
        }));
    Ok((source, inst))
}

pub fn parse_landingpad<'a, 'b>(
    source: &'a str,
    ctx: &mut ParserContext<'b>,
) -> IResult<&'a str, Instruction, VerboseError<&'a str>> {
    let (source, _) = preceded(spaces, tag("landingpad"))(source)?;
    let (source, ty) = super::types::parse(ctx.types)(source)?;
    let (mut source, cleanup) = opt(preceded(spaces, tag("cleanup")))(source)?;
    let mut catches = vec![];
    loop {
        let (source_, catch) = opt(preceded(spaces, tag("catch")))(source)?;
        if catch.is_none() {
            break;
        }
        let (source_, ty) = super::types::parse(ctx.types)(source_)?;
        let (source_, arg) = super::value::parse(source_, ctx, ty)?;
        catches.push((ty, arg));
        source = source_;
    }
    assert!(cleanup.is_some() || (cleanup.is_none() && catches.len() > 0));
    let inst = Opcode::LandingPad
        .with_block(ctx.cur_block)
        .with_operand(Operand::LandingPad(LandingPad {
            ty,
            catches,
            cleanup: cleanup.is_some(),
        }));
    Ok((source, inst))
}

pub fn parse_resume<'a, 'b>(
    source: &'a str,
    ctx: &mut ParserContext<'b>,
) -> IResult<&'a str, Instruction, VerboseError<&'a str>> {
    let (source, _) = preceded(spaces, tag("resume"))(source)?;
    let (source, ty) = super::types::parse(ctx.types)(source)?;
    let (source, arg) = super::value::parse(source, ctx, ty)?;
    let inst = Opcode::Resume
        .with_block(ctx.cur_block)
        .with_operand(Operand::Resume(Resume { ty, arg }));
    Ok((source, inst))
}

pub fn parse_br<'a, 'b>(
    source: &'a str,
    ctx: &mut ParserContext<'b>,
) -> IResult<&'a str, Instruction, VerboseError<&'a str>> {
    let (source, _) = preceded(spaces, tag("br"))(source)?;
    if let Ok((source, _)) = preceded(spaces, tag("label"))(source) {
        let (source, label) = preceded(spaces, preceded(char('%'), super::name::parse))(source)?;
        let block = ctx.get_or_create_named_block(label);
        let inst = Opcode::Br
            .with_block(ctx.cur_block)
            .with_operand(Operand::Br(Br { block }));
        Ok((source, inst))
    } else {
        let (source, ty) = super::types::parse(ctx.types)(source)?;
        assert!(ty.is_i1());
        let (source, arg) = super::value::parse(source, ctx, ty)?;
        let (source, _) = preceded(spaces, char(','))(source)?;
        let (source, iftrue) = preceded(
            spaces,
            preceded(
                tag("label"),
                preceded(spaces, preceded(char('%'), super::name::parse)),
            ),
        )(source)?;
        let (source, _) = preceded(spaces, char(','))(source)?;
        let (source, iffalse) = preceded(
            spaces,
            preceded(
                tag("label"),
                preceded(spaces, preceded(char('%'), super::name::parse)),
            ),
        )(source)?;
        let iftrue = ctx.get_or_create_named_block(iftrue);
        let iffalse = ctx.get_or_create_named_block(iffalse);
        let inst = Opcode::CondBr
            .with_block(ctx.cur_block)
            .with_operand(Operand::CondBr(CondBr {
                arg,
                blocks: [iftrue, iffalse],
            }));
        Ok((source, inst))
    }
}

pub fn parse_switch<'a, 'b>(
    source: &'a str,
    ctx: &mut ParserContext<'b>,
) -> IResult<&'a str, Instruction, VerboseError<&'a str>> {
    let (source, _) = preceded(spaces, tag("switch"))(source)?;
    let (source, cond_ty) = super::types::parse(ctx.types)(source)?;
    let (source, cond) = super::value::parse(source, ctx, cond_ty)?;
    let (source, _) = preceded(spaces, char(','))(source)?;
    let (source, default_block) = preceded(
        spaces,
        preceded(
            tag("label"),
            preceded(spaces, preceded(char('%'), super::name::parse)),
        ),
    )(source)?;
    let default_block = ctx.get_or_create_named_block(default_block);
    let mut tys = vec![cond_ty];
    let mut args = vec![cond];
    let mut blocks = vec![default_block];
    let (mut source, _) = preceded(spaces, char('['))(source)?;
    loop {
        let (source_, end) = opt(preceded(spaces, char(']')))(source)?;
        if end.is_some() {
            source = source_;
            break;
        }
        let (source_, case_ty) = super::types::parse(ctx.types)(source_)?;
        assert!(case_ty == cond_ty);
        let (source_, case) = super::value::parse(source_, ctx, case_ty)?;
        let (source_, _) = preceded(spaces, char(','))(source_)?;
        let (source_, block) = preceded(
            spaces,
            preceded(
                tag("label"),
                preceded(spaces, preceded(char('%'), super::name::parse)),
            ),
        )(source_)?;
        let block = ctx.get_or_create_named_block(block);
        tys.push(case_ty);
        args.push(case);
        blocks.push(block);
        source = source_;
    }
    let inst = Opcode::Switch
        .with_block(ctx.cur_block)
        .with_operand(Operand::Switch(Switch { tys, args, blocks }));
    Ok((source, inst))
}

pub fn parse_ret<'a, 'b>(
    source: &'a str,
    ctx: &mut ParserContext<'b>,
) -> IResult<&'a str, Instruction, VerboseError<&'a str>> {
    let (source, _) = preceded(spaces, preceded(tag("ret"), spaces))(source)?;
    let (source, ty) = super::types::parse(ctx.types)(source)?;
    let (source, val) = if ty.is_void() {
        (source, None)
    } else {
        let (source, val) = super::value::parse(source, ctx, ty)?;
        (source, Some(val))
    };
    Ok((
        source,
        Opcode::Ret
            .with_block(ctx.cur_block)
            .with_operand(Operand::Ret(Ret { val, ty })),
    ))
}

pub fn parse_unreachable<'a, 'b>(
    source: &'a str,
    ctx: &mut ParserContext<'b>,
) -> IResult<&'a str, Instruction, VerboseError<&'a str>> {
    let (source, _) = preceded(spaces, preceded(tag("unreachable"), spaces))(source)?;
    Ok((
        source,
        Opcode::Unreachable
            .with_block(ctx.cur_block)
            .with_operand(Operand::Unreachable),
    ))
}

fn parse_metadata<'a>(
    name: &'static str,
) -> impl Fn(&'a str) -> IResult<&'a str, &'a str, VerboseError<&'a str>> {
    move |source: &'a str| {
        let (source, (_, _, _, _, _, _, num)) = tuple((
            spaces,
            char(','),
            spaces,
            tag(name),
            spaces,
            char('!'),
            digit1,
        ))(source)?;
        Ok((source, num))
    }
}

fn parse_metadata_if_any(
    types: &types::Types,
) -> impl Fn(&str) -> IResult<&str, FxHashMap<String, Metadata>, VerboseError<&str>> + '_ {
    move |mut source| {
        let mut metadata = FxHashMap::default();
        loop {
            match preceded(spaces, char(','))(source) {
                Ok((src, _)) => source = src,
                Err(_) => return Ok((source, metadata)),
            }
            let (src, kind) = preceded(spaces, preceded(char('!'), identifier))(source)?;
            let (src, meta) = super::metadata::operand(types)(src)?;
            metadata.insert(kind.to_owned(), meta);
            source = src;
        }
    }
}

/// Only parses `source` as Instruction. Doesn't append instruction to block.
pub fn parse<'a, 'b>(
    source: &'a str,
    ctx: &mut ParserContext<'b>,
) -> IResult<&'a str, InstructionId, VerboseError<&'a str>> {
    let (source, name) = opt(tuple((
        spaces,
        char('%'),
        super::name::parse,
        spaces,
        char('='),
    )))(source)?;
    let name = name.map(|(_, _, name, _, _)| name);
    for f in [
        parse_alloca,
        parse_phi,
        parse_load,
        parse_store,
        parse_insertvalue,
        parse_extractvalue,
        parse_add_sub_mul,
        parse_icmp,
        parse_cast,
        parse_getelementptr,
        parse_call,
        parse_invoke,
        parse_landingpad,
        parse_resume,
        parse_br,
        parse_switch,
        parse_ret,
        parse_unreachable,
    ]
    .iter()
    {
        if let Ok((source, mut inst)) = f(source, ctx) {
            let (source, metadata) = parse_metadata_if_any(ctx.types)(source)?;
            inst = inst.with_metadata(metadata);

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
