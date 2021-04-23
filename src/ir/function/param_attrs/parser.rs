use super::ParameterAttribute;
use crate::ir::util::{spaces, string_literal};
use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::{char, digit1},
    combinator::map,
    error::VerboseError,
    multi::many0,
    sequence::{preceded, tuple},
    IResult,
};

pub fn parse_param_attr<'a>(
    source: &'a str,
) -> IResult<&'a str, ParameterAttribute, VerboseError<&'a str>> {
    alt((
        map(tag("zeroext"), |_| ParameterAttribute::ZeroExt),
        map(tag("signext"), |_| ParameterAttribute::SignExt),
        map(tag("inreg"), |_| ParameterAttribute::InReg),
        map(tag("byval"), |_| ParameterAttribute::ByVal),
        map(tag("inalloca"), |_| ParameterAttribute::InAlloca),
        map(tag("sret"), |_| ParameterAttribute::SRet),
        // map(tag("alignment"), |_| ParameterAttribute::SRet),
        map(tag("noalias"), |_| ParameterAttribute::NoAlias),
        map(tag("nocapture"), |_| ParameterAttribute::NoCapture),
        map(tag("nofree"), |_| ParameterAttribute::NoFree),
        map(tag("nest"), |_| ParameterAttribute::Nest),
        map(tag("returned"), |_| ParameterAttribute::Returned),
        map(tag("nonnull"), |_| ParameterAttribute::NonNull),
        // map(tag("dereferenceableornull"), |_| ParameterAttribute::SRet),
        // map(tag("dereferenceable"), |_| ParameterAttribute::SRet),
        map(tag("swiftself"), |_| ParameterAttribute::SwiftSelf),
        map(tag("swifterror"), |_| ParameterAttribute::SwiftError),
        map(tag("immarg"), |_| ParameterAttribute::ImmArg),
        map(
            tuple((string_literal, spaces, char('='), spaces, string_literal)),
            |(kind, _, _, _, value)| ParameterAttribute::StringAttribute {
                kind: kind.to_string(),
                value: value.to_string(),
            },
        ),
        map(preceded(char('#'), digit1), |num: &str| {
            ParameterAttribute::Ref(num.parse::<u32>().unwrap())
        }),
    ))(source)
}

pub fn parse_param_attrs<'a>(
    source: &'a str,
) -> IResult<&'a str, Vec<ParameterAttribute>, VerboseError<&'a str>> {
    many0(preceded(spaces, parse_param_attr))(source)
}
