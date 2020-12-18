use super::super::{
    function::{Data, Function, Layout, Parameter},
    module::preemption_specifier,
    types,
    types::Types,
    util::spaces,
};
use nom::{
    bytes::complete::tag,
    character::complete::{alphanumeric1, char},
    combinator::opt,
    error::VerboseError,
    sequence::{preceded, tuple},
    IResult,
};

// define [linkage] [PreemptionSpecifier] [visibility] [DLLStorageClass]
//        [cconv] [ret attrs]
//        <ResultType> @<FunctionName> ([argument list])
//        [(unnamed_addr|local_unnamed_addr)] [AddrSpace] [fn Attrs]
//        [section "name"] [comdat [($name)]] [align N] [gc] [prefix Constant]
//        [prologue Constant] [personality Constant] (!name !N)* { ... }

pub fn parse_argument_list<'a>(
    source: &'a str,
    types: &Types,
) -> IResult<&'a str, Vec<Parameter>, VerboseError<&'a str>> {
    todo!()
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
    let (source, _) = tuple((spaces, char('('), spaces))(source)?;
    // argument_list, spaces, char(')')));

    Ok((
        source,
        Function {
            name: name.to_string(),
            is_var_arg: false,
            result_ty,
            preemption_specifier: preemption_specifier
                .unwrap_or(preemption_specifier::PreemptionSpecifier::DsoPreemptable),
            params: vec![],
            data: Data::new(),
            layout: Layout::new(),
            types,
        },
    ))
}

#[test]
fn parse_function1() {
    let types = Types::new();
    let result = parse(
        r#"
        define dso_local i32 @main()
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
}
