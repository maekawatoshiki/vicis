use super::super::{
    basic_block::BasicBlockId,
    function::{Data, Function, Layout, Parameter},
    instruction,
    module::{attributes, name, preemption_specifier},
    types,
    types::Types,
    util::spaces,
    value::{Value, ValueId},
};
use either::Either;
use nom::{
    bytes::complete::tag,
    character::complete::{alphanumeric1, char, digit1},
    combinator::opt,
    error::VerboseError,
    sequence::{preceded, terminated, tuple},
    IResult,
};
use rustc_hash::FxHashMap;

// define [linkage] [PreemptionSpecifier] [visibility] [DLLStorageClass]
//        [cconv] [ret attrs]
//        <ResultType> @<FunctionName> ([argument list])
//        [(unnamed_addr|local_unnamed_addr)] [AddrSpace] [fn Attrs]
//        [section "name"] [comdat [($name)]] [align N] [gc] [prefix Constant]
//        [prologue Constant] [personality Constant] (!name !N)* { ... }

pub struct ParserContext<'a> {
    pub types: &'a Types,
    pub data: &'a mut Data,
    pub layout: &'a mut Layout,
    pub name_to_value: &'a mut FxHashMap<name::Name, ValueId>,
    pub cur_block: BasicBlockId,
}

pub fn parse_argument<'a>(
    source: &'a str,
    types: &Types,
) -> IResult<&'a str, Parameter, VerboseError<&'a str>> {
    let (source, ty) = types::parse(source, types)?;
    let (source, _) = preceded(spaces, char('%'))(source)?;
    let (source, name) = name::parse(source)?;
    Ok((source, Parameter { name, ty }))
}

pub fn parse_argument_list<'a>(
    source: &'a str,
    types: &Types,
) -> IResult<&'a str, Vec<Parameter>, VerboseError<&'a str>> {
    let (mut source, _) = tuple((spaces, char('(')))(source)?;

    if let Ok((source, _)) = tuple((spaces, char(')')))(source) {
        return Ok((source, vec![]));
    }

    let mut params = vec![];

    loop {
        let (source_, param) = parse_argument(source, types)?;
        params.push(param);

        if let Ok((source_, _)) = tuple((spaces, char(',')))(source_) {
            source = source_;
            continue;
        }

        if let Ok((source, _)) = tuple((spaces, char(')')))(source_) {
            return Ok((source, params));
        }
    }
}
//
pub fn parse_func_attrs<'a>(
    mut source: &'a str,
) -> IResult<&'a str, Vec<Either<attributes::Attribute, u32>>, VerboseError<&'a str>> {
    let mut attrs = vec![];
    loop {
        if let Ok((source_, num)) = preceded(spaces, preceded(char('#'), digit1))(source) {
            attrs.push(Either::Right(num.parse::<u32>().unwrap()));
            source = source_;
            continue;
        }
        if let Ok((source_, attr)) = preceded(spaces, attributes::parser::parse_attribute)(source) {
            attrs.push(Either::Left(attr));
            source = source_;
        }
        break;
    }
    Ok((source, attrs))
}

pub fn parse_body<'a, 'b>(
    source: &'a str,
    ctx: &mut ParserContext<'b>,
) -> IResult<&'a str, (), VerboseError<&'a str>> {
    let (mut source, _) = tuple((spaces, char('{')))(source)?;

    if let Ok((source, _)) = tuple((spaces, char('}')))(source) {
        return Ok((source, ()));
    }

    let mut label_to_block = FxHashMap::default();

    // Parse each block
    loop {
        // Parse label if any
        let (mut source_, label) = opt(preceded(
            spaces,
            terminated(name::parse, preceded(spaces, char(':'))),
        ))(source)?;
        debug!(&label);

        let block = ctx.data.create_block();
        ctx.layout.append_block(block);
        ctx.cur_block = block;

        // If label is present, set it to block
        if let Some(label) = label {
            label_to_block.insert(label.clone(), block);
            ctx.data.block_ref_mut(block).name = Some(label);
        }

        while let Ok((source__, inst)) = instruction::parse(source_, ctx) {
            ctx.layout.append_inst(inst, ctx.cur_block);
            source_ = source__
        }

        if let Ok((source, _)) = tuple((spaces, char('}')))(source_) {
            return Ok((source, ()));
        }

        source = source_
    }
}

pub fn parse<'a>(
    source: &'a str,
    types: Types,
) -> IResult<&'a str, Function, VerboseError<&'a str>> {
    let (source, _) = preceded(spaces, tag("define"))(source)?;
    let (source, preemption_specifier) =
        opt(preceded(spaces, preemption_specifier::parse))(source)?;
    let (source, result_ty) = types::parse(source, &types)?;
    let (source, (_, _, _, name)) = tuple((spaces, char('@'), spaces, alphanumeric1))(source)?;
    let (source, params) = parse_argument_list(source, &types)?;
    let (source, fn_attrs) = parse_func_attrs(source)?;

    let mut data = Data::new();
    let mut layout = Layout::new();
    let mut name_to_value = FxHashMap::default();
    let dummy_block = data.create_block();

    for (i, param) in params.iter().enumerate() {
        let arg = data.create_value(Value::Argument(i));
        name_to_value.insert(param.name.clone(), arg);
    }

    let (source, _) = parse_body(
        source,
        &mut ParserContext {
            types: &types,
            data: &mut data,
            layout: &mut layout,
            name_to_value: &mut name_to_value,
            cur_block: dummy_block,
        },
    )?;

    Ok((
        source,
        Function {
            name: name.to_string(),
            is_var_arg: false,
            result_ty,
            preemption_specifier: preemption_specifier
                .unwrap_or(preemption_specifier::PreemptionSpecifier::DsoPreemptable),
            attributes: fn_attrs,
            params,
            data,
            layout,
            types,
        },
    ))
}

#[test]
fn test_parse_function1() {
    let types = Types::new();
    let result = parse(
        r#"
        define dso_local i32 @main(i32 %0, i32 %1) {
            ret i32 0
        }
        "#,
        types,
    );
    assert!(result.is_ok());
    let result = result.unwrap().1;
    assert_eq!(result.name, "main");
    assert_eq!(
        result.preemption_specifier,
        preemption_specifier::PreemptionSpecifier::DsoLocal
    );
    println!("{:?}", result);
}

#[test]
fn test_parse_function2() {
    let types = Types::new();
    let result = parse(
        r#"
        define dso_local i32 @main() #0 noinline {
        entry:
            %1 = alloca i32, align 4
            store i32 1, i32* %1
            ret i32 0
        }
        "#,
        types,
    );
    assert!(result.is_ok());
    let result = result.unwrap().1;
    assert_eq!(result.name, "main");
    assert_eq!(
        result.preemption_specifier,
        preemption_specifier::PreemptionSpecifier::DsoLocal
    );
    println!("{:?}", result);
}
