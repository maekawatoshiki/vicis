use super::super::attributes::{parser::parse_attributes, Attribute};
use super::Module;
use nom::{
    branch::alt,
    bytes::complete::{tag, take_until},
    character::complete::{char, digit1, multispace0},
    combinator::{cut, map},
    error::VerboseError,
    multi::many1,
    sequence::{preceded, terminated, tuple},
    Err, IResult,
};
use rustc_hash::FxHashMap;

enum ParsedElement<'a> {
    SourceFilename(&'a str),
    TargetDatalayout(&'a str),
    TargetTriple(&'a str),
    AttributeGroup(u32, Vec<Attribute>),
    Metadata,
}

pub fn spaces<'a>(source: &'a str) -> IResult<&'a str, (), VerboseError<&'a str>> {
    alt((
        map(
            many1(tuple((
                multispace0,
                preceded(char(';'), terminated(take_until("\n"), char('\n'))),
                multispace0,
            ))),
            |_| (),
        ),
        map(multispace0, |_| ()),
    ))(source)
}

pub fn parse_string_literal<'a>(
    source: &'a str,
) -> IResult<&'a str, &'a str, VerboseError<&'a str>> {
    preceded(char('\"'), cut(terminated(take_until("\""), char('\"'))))(source)
}

fn parse_source_filename<'a>(
    source: &'a str,
) -> IResult<&'a str, ParsedElement<'a>, VerboseError<&'a str>> {
    tuple((
        tag("source_filename"),
        preceded(spaces, char('=')),
        preceded(spaces, parse_string_literal),
    ))(source)
    .map(|(i, (_, _, name))| (i, ParsedElement::SourceFilename(name)))
}

fn parse_target_datalayout<'a>(
    source: &'a str,
) -> IResult<&'a str, ParsedElement<'a>, VerboseError<&'a str>> {
    tuple((
        tag("target"),
        preceded(spaces, tag("datalayout")),
        preceded(spaces, char('=')),
        preceded(spaces, parse_string_literal),
    ))(source)
    .map(|(i, (_, _, _, datalayout))| (i, ParsedElement::TargetDatalayout(datalayout)))
}

fn parse_target_triple<'a>(
    source: &'a str,
) -> IResult<&'a str, ParsedElement<'a>, VerboseError<&'a str>> {
    tuple((
        tag("target"),
        preceded(spaces, tag("triple")),
        preceded(spaces, char('=')),
        preceded(spaces, parse_string_literal),
    ))(source)
    .map(|(i, (_, _, _, triple))| (i, ParsedElement::TargetTriple(triple)))
}

fn parse_attribute_group<'a>(
    source: &'a str,
) -> IResult<&'a str, ParsedElement<'a>, VerboseError<&'a str>> {
    tuple((
        tag("attributes"),
        preceded(spaces, char('#')),
        digit1,
        preceded(spaces, char('=')),
        preceded(spaces, char('{')),
        preceded(spaces, parse_attributes),
        preceded(spaces, char('}')),
    ))(source)
    .map(|(i, (_, _, id, _, _, attrs, _))| {
        (i, ParsedElement::AttributeGroup(id.parse().unwrap(), attrs))
    })
}

fn parse_metadata<'a>(
    source: &'a str,
) -> IResult<&'a str, ParsedElement<'a>, VerboseError<&'a str>> {
    map(
        preceded(char('!'), terminated(take_until("\n"), char('\n'))),
        |_| ParsedElement::Metadata,
    )(source)
}

pub fn parse_module<'a>(mut source: &'a str) -> Result<Module, Err<VerboseError<&'a str>>> {
    let mut module = Module::new();
    let mut attr_groups: FxHashMap<u32, Vec<Attribute>> = FxHashMap::default();

    loop {
        let (source_, (_, element, _)) = tuple((
            spaces,
            alt((
                parse_source_filename,
                parse_target_datalayout,
                parse_target_triple,
                parse_attribute_group,
                parse_metadata,
            )),
            spaces,
        ))(source)?;

        match element {
            ParsedElement::SourceFilename(name) => {
                module.source_filename = name.to_string();
            }
            ParsedElement::TargetDatalayout(datalayout) => {
                module.target.datalayout = datalayout.to_string();
            }
            ParsedElement::TargetTriple(triple) => {
                module.target.triple = triple.to_string();
            }
            ParsedElement::AttributeGroup(id, attrs) => {
                attr_groups.insert(id, attrs);
            }
            ParsedElement::Metadata => {}
        }

        if source_.is_empty() {
            break;
        }
        source = source_
    }

    // println!("{:?}", attr_groups);

    Ok(module)
}

#[test]
fn parse1_module() {
    let result = parse_module(
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
    println!("{:?}", result);
    assert!(result.is_ok());
    let result = result.unwrap();
    assert_eq!(result.source_filename, "c.c");
    assert_eq!(
        result.target.datalayout,
        "e-m:e-p270:32:32-p271:32:32-p272:64:64-i64:64-f80:128-n8:16:32:64-S128"
    );
    assert_eq!(result.target.triple, "x86_64-pc-linux-gnu");
}
