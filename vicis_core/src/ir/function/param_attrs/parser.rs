use super::ParameterAttribute;
use crate::ir::{
    types,
    util::{spaces, string_literal},
};
use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::{char, digit1},
    combinator::{map, opt},
    error::VerboseError,
    multi::many0,
    sequence::{preceded, tuple},
    IResult,
};

pub fn parse_param_attr<'a>(
    source: &'a str,
    types: &types::Types,
) -> IResult<&'a str, ParameterAttribute, VerboseError<&'a str>> {
    alt((
        map(tag("zeroext"), |_| ParameterAttribute::ZeroExt),
        map(tag("signext"), |_| ParameterAttribute::SignExt),
        map(tag("inreg"), |_| ParameterAttribute::InReg),
        map(tag("byval"), |_| ParameterAttribute::ByVal),
        map(tag("inalloca"), |_| ParameterAttribute::InAlloca),
        map(
            tuple((
                tag("sret"),
                spaces,
                char('('),
                |source: &'a str| types::parser::parse(source, types),
                char(')'),
            )),
            |(_, _, _, ty, _)| ParameterAttribute::SRet(Some(ty)),
        ),
        map(tag("sret"), |_| ParameterAttribute::SRet(None)),
        map(
            tuple((tag("align"), spaces, opt(char('(')), digit1, opt(char(')')))),
            |(_, _, _, num, _)| ParameterAttribute::Alignment(num.parse::<u64>().unwrap()),
        ),
        map(tag("readonly"), |_| ParameterAttribute::ReadOnly),
        map(tag("noalias"), |_| ParameterAttribute::NoAlias),
        map(tag("nocapture"), |_| ParameterAttribute::NoCapture),
        map(tag("nofree"), |_| ParameterAttribute::NoFree),
        map(tag("nest"), |_| ParameterAttribute::Nest),
        map(tag("returned"), |_| ParameterAttribute::Returned),
        map(tag("nonnull"), |_| ParameterAttribute::NonNull),
        // map(tag("dereferenceableornull"), |_| ParameterAttribute::SRet),
        map(
            tuple((
                tag("dereferenceable"),
                spaces,
                opt(char('(')),
                digit1,
                opt(char(')')),
            )),
            |(_, _, _, num, _): (_, _, _, &'a str, _)| {
                ParameterAttribute::Dereferenceable(num.parse::<u64>().unwrap())
            },
        ),
        map(tag("swiftself"), |_| ParameterAttribute::SwiftSelf),
        map(tag("swifterror"), |_| ParameterAttribute::SwiftError),
        map(tag("writeonly"), |_| ParameterAttribute::WriteOnly),
        map(tag("immarg"), |_| ParameterAttribute::ImmArg),
        alt((
            map(
                tuple((string_literal, spaces, char('='), spaces, string_literal)),
                |(kind, _, _, _, value)| ParameterAttribute::StringAttribute { kind, value },
            ),
            map(preceded(char('#'), digit1), |num: &str| {
                ParameterAttribute::Ref(num.parse::<u32>().unwrap())
            }),
        )),
    ))(source)
}

pub fn parse_param_attrs<'a>(
    source: &'a str,
    types: &types::Types,
) -> IResult<&'a str, Vec<ParameterAttribute>, VerboseError<&'a str>> {
    many0(preceded(spaces, |source: &'a str| {
        parse_param_attr(source, types)
    }))(source)
}
