use super::super::{
    basic_block::BasicBlockId,
    function::{Data, Function, Layout, Parameter},
    instruction,
    module::{name, preemption_specifier},
    types,
    types::Types,
    util::spaces,
};
use nom::{
    bytes::complete::tag,
    character::complete::{alphanumeric1, char},
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

pub fn parse_body<'a>(
    source: &'a str,
    types: &Types,
) -> IResult<&'a str, (Data, Layout), VerboseError<&'a str>> {
    let (source, _) = tuple((spaces, char('{')))(source)?;

    let mut data = Data::new();
    let mut layout = Layout::new();

    if let Ok((source, _)) = tuple((spaces, char('}')))(source) {
        return Ok((source, (data, layout)));
    }

    let mut label_to_block = FxHashMap::default();

    // Parse each block
    loop {
        let (source_, label) = opt(preceded(
            spaces,
            terminated(alphanumeric1, preceded(spaces, char(':'))),
        ))(source)?;

        debug!(label);

        let block = data.create_block();
        layout.append_block(block);

        if let Some(label) = label {
            label_to_block.insert(label, block);
        }

        let mut ctx = ParserContext {
            types,
            data: &mut data,
            layout: &mut layout,
            cur_block: block,
        };

        let (source_, _) = instruction::parse(source_, &mut ctx)?;

        if let Ok((source, _)) = tuple((spaces, char('}')))(source_) {
            return Ok((source, (data, layout)));
        }
    }
}

pub fn parse<'a>(
    source: &'a str,
    types: Types,
) -> IResult<&'a str, Function, VerboseError<&'a str>> {
    let (source, _) = preceded(spaces, tag("define"))(source)?;
    let (source, preemption_specifier) =
        opt(preceded(spaces, preemption_specifier::parse))(source)?;
    debug!(preemption_specifier);
    let (source, result_ty) = types::parse(source, &types)?;
    let (source, (_, _, _, name)) = tuple((spaces, char('@'), spaces, alphanumeric1))(source)?;
    let (source, params) = parse_argument_list(source, &types)?;
    let (source, (data, layout)) = parse_body(source, &types)?;

    Ok((
        source,
        Function {
            name: name.to_string(),
            is_var_arg: false,
            result_ty,
            preemption_specifier: preemption_specifier
                .unwrap_or(preemption_specifier::PreemptionSpecifier::DsoPreemptable),
            params,
            data,
            layout,
            types,
        },
    ))
}

#[test]
fn test_parse_function() {
    let types = Types::new();
    let result = parse(
        r#"
        define dso_local i32 @main(i32 %0, i32 %1) {
        entry:
            ret void
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
