use super::super::attributes::{parser::parse_attributes, Attribute};
use super::super::parser::spaces;
use super::Module;
use nom::{
    bytes::complete::{tag, take_until},
    character::complete::{char, digit1},
    combinator::{cut, map},
    error::VerboseError,
    sequence::{preceded, terminated, tuple},
    Err, IResult,
};
use rustc_hash::FxHashMap;

pub fn parse_string_literal<'a>(
    source: &'a str,
) -> IResult<&'a str, &'a str, VerboseError<&'a str>> {
    preceded(char('\"'), cut(terminated(take_until("\""), char('\"'))))(source)
}

fn parse_source_filename<'a>(source: &'a str) -> IResult<&'a str, &'a str, VerboseError<&'a str>> {
    tuple((
        tag("source_filename"),
        preceded(spaces, char('=')),
        preceded(spaces, parse_string_literal),
    ))(source)
    .map(|(i, (_, _, name))| (i, name))
}

fn parse_target_datalayout<'a>(
    source: &'a str,
) -> IResult<&'a str, &'a str, VerboseError<&'a str>> {
    tuple((
        tag("target"),
        preceded(spaces, tag("datalayout")),
        preceded(spaces, char('=')),
        preceded(spaces, parse_string_literal),
    ))(source)
    .map(|(i, (_, _, _, datalayout))| (i, datalayout))
}

fn parse_target_triple<'a>(source: &'a str) -> IResult<&'a str, &'a str, VerboseError<&'a str>> {
    tuple((
        tag("target"),
        preceded(spaces, tag("triple")),
        preceded(spaces, char('=')),
        preceded(spaces, parse_string_literal),
    ))(source)
    .map(|(i, (_, _, _, triple))| (i, triple))
}

fn parse_attribute_group<'a>(
    source: &'a str,
) -> IResult<&'a str, (u32, Vec<Attribute>), VerboseError<&'a str>> {
    tuple((
        tag("attributes"),
        preceded(spaces, char('#')),
        digit1,
        preceded(spaces, char('=')),
        preceded(spaces, char('{')),
        preceded(spaces, parse_attributes),
        preceded(spaces, char('}')),
    ))(source)
    .map(|(i, (_, _, id, _, _, attrs, _))| (i, (id.parse().unwrap(), attrs)))
}

fn parse_metadata<'a>(source: &'a str) -> IResult<&'a str, (), VerboseError<&'a str>> {
    map(
        preceded(char('!'), terminated(take_until("\n"), char('\n'))),
        |_| (),
    )(source)
}

pub fn parse<'a>(mut source: &'a str) -> Result<Module, Err<VerboseError<&'a str>>> {
    let mut module = Module::new();
    let mut attr_groups: FxHashMap<u32, Vec<Attribute>> = FxHashMap::default();

    loop {
        source = spaces(source)?.0;

        if source.is_empty() {
            break;
        }

        if let Ok((source_, source_filename)) = parse_source_filename(source) {
            module.source_filename = source_filename.to_string();
            source = source_;
            continue;
        }

        if let Ok((source_, target_datalayout)) = parse_target_datalayout(source) {
            module.target.datalayout = target_datalayout.to_string();
            source = source_;
            continue;
        }

        if let Ok((source_, target_triple)) = parse_target_triple(source) {
            module.target.triple = target_triple.to_string();
            source = source_;
            continue;
        }

        if let Ok((source_, (id, attrs))) = parse_attribute_group(source) {
            attr_groups.insert(id, attrs);
            source = source_;
            continue;
        }

        if let Ok((source_, _)) = parse_metadata(source) {
            source = source_;
            continue;
        }
    }

    debug!(attr_groups);

    Ok(module)
}

#[test]
fn parse_module1() {
    let result = parse(
        r#"
            source_filename = "c.c"
            target datalayout = "e-m:e-p270:32:32-p271:32:32-p272:64:64-i64:64-f80:128-n8:16:32:64-S128"
            ; comments
            ; bluh bluh
            target triple = "x86_64-pc-linux-gnu" ; hogehoge
            !0 = {}
            attributes #0 = { noinline "abcde" = ;fff
            ;ff
                                            "ff"}
        "#,
    );
    debug!(&result);
    assert!(result.is_ok());
    let result = result.unwrap();
    assert_eq!(result.source_filename, "c.c");
    assert_eq!(
        result.target.datalayout,
        "e-m:e-p270:32:32-p271:32:32-p272:64:64-i64:64-f80:128-n8:16:32:64-S128"
    );
    assert_eq!(result.target.triple, "x86_64-pc-linux-gnu");
}
