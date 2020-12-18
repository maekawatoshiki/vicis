use super::super::{
    function::{Data, Function, Layout},
    parser::spaces,
    preemption_specifier, types,
    types::Types,
};
use nom::{
    bytes::complete::tag, combinator::opt, error::VerboseError, sequence::preceded, IResult,
};

// define [linkage] [PreemptionSpecifier] [visibility] [DLLStorageClass]
//        [cconv] [ret attrs]
//        <ResultType> @<FunctionName> ([argument list])
//        [(unnamed_addr|local_unnamed_addr)] [AddrSpace] [fn Attrs]
//        [section "name"] [comdat [($name)]] [align N] [gc] [prefix Constant]
//        [prologue Constant] [personality Constant] (!name !N)* { ... }

pub fn parse<'a>(
    source: &'a str,
    types: Types,
) -> IResult<&'a str, Function, VerboseError<&'a str>> {
    let (source, _) = preceded(spaces, tag("define"))(source)?;
    let (source, preemption_specifier) =
        opt(preceded(spaces, preemption_specifier::parse))(source)?;
    debug!(preemption_specifier);
    let (source, result_ty) = types::parse(source, &types)?;
    // preceded(spaces, types::parse)(source)?;

    Ok((
        source,
        Function {
            name: "".to_string(),
            is_var_arg: false,
            result_ty,
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
        define dso_local i32
        "#,
        types,
    );
    // println!("{:?}", result);
    assert!(result.is_ok());
    // let result = result.unwrap();
}
