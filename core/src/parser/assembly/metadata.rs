use super::value::parse_constant_int;
use crate::ir::module::{metadata::Metadata, name::Name};
use crate::ir::{
    types,
    util::{spaces, string_literal},
};
use nom::{
    self,
    branch::alt,
    bytes::complete::tag,
    character::complete::char,
    error::VerboseError,
    multi::separated_list0,
    sequence::{preceded, separated_pair, tuple},
    IResult,
};

pub fn parse(
    types: &types::Types,
) -> impl Fn(&str) -> IResult<&str, (Name, Metadata), VerboseError<&str>> + '_ {
    move |source| {
        separated_pair(
            preceded(exclamation, super::name::parse),
            preceded(spaces, tag("=")),
            node(types),
        )(source)
    }
}

fn exclamation(source: &str) -> IResult<&str, &str, VerboseError<&str>> {
    preceded(spaces, tag("!"))(source)
}

fn string(source: &str) -> IResult<&str, Metadata, VerboseError<&str>> {
    preceded(exclamation, preceded(spaces, string_literal))(source)
        .map(|(i, source)| (i, Metadata::String(source)))
}

fn name(source: &str) -> IResult<&str, Metadata, VerboseError<&str>> {
    preceded(exclamation, super::name::parse)(source).map(|(i, source)| (i, Metadata::Name(source)))
}

fn int(types: &types::Types) -> impl Fn(&str) -> IResult<&str, Metadata, VerboseError<&str>> + '_ {
    move |source| {
        let (source, ty) = super::types::parse(types)(source)?;
        parse_constant_int(source, ty).map(|(source, v)| (source, Metadata::Int(v)))
    }
}

fn node(types: &types::Types) -> impl Fn(&str) -> IResult<&str, Metadata, VerboseError<&str>> + '_ {
    move |source| {
        tuple((
            exclamation,
            spaces,
            char('{'),
            separated_list0(preceded(spaces, tag(",")), operand(types)),
            spaces,
            char('}'),
        ))(source)
        .map(|(i, (_, _, _, list, _, _))| (i, Metadata::Node(list)))
    }
}

pub fn operand(
    types: &types::Types,
) -> impl Fn(&str) -> IResult<&str, Metadata, VerboseError<&str>> + '_ {
    move |source| alt((string, name, node(types), int(types)))(source)
}

#[test]
fn test1() {
    use crate::ir::types::Types;
    use nom::multi::many1;

    let source = "
        !llvm.module.flags = !{!0}
        !llvm.ident = !{!1}
        !0 = !{i32 1, !\"wchar_size\", i32 4}
        !1 = !{!\"clang version 10.0.0-4ubuntu1 \"}
        !llvm.module.flags = !{!0, !1, !2}
        !0 = !{i32 7, !\"PIC Level\", i32 2}
        !1 = !{i32 7, !\"PIE Level\", i32 2}
        !2 = !{i32 2, !\"RtLibUseGOT\", i32 1}
        !3 = !{}
        !4 = !{i32 2849348}
        !4 = !{i32 2849319}
        !4 = !{i32 2849383}";

    insta::assert_debug_snapshot!(many1(parse(&Types::new()))(source));
}
