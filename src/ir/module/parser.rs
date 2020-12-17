use super::Module;
use nom::{
    branch::alt,
    bytes::complete::{tag, take_until},
    character::complete::{char, multispace0, space0},
    combinator::{cut, map},
    error::VerboseError,
    sequence::{preceded, terminated, tuple},
    Err, IResult,
};

enum ModuleElement<'a> {
    SourceFilename(&'a str),
    TargetDatalayout(&'a str),
    TargetTriple(&'a str),
    Comment,
}

fn parse_string_literal<'a>(source: &'a str) -> IResult<&'a str, &'a str, VerboseError<&'a str>> {
    preceded(char('\"'), cut(terminated(take_until("\""), char('\"'))))(source)
}

fn parse_source_filename<'a>(
    source: &'a str,
) -> IResult<&'a str, ModuleElement<'a>, VerboseError<&'a str>> {
    tuple((
        tag("source_filename"),
        preceded(space0, char('=')),
        preceded(space0, parse_string_literal),
    ))(source)
    .map(|(i, (_, _, name))| (i, ModuleElement::SourceFilename(name)))
}

fn parse_target_datalayout<'a>(
    source: &'a str,
) -> IResult<&'a str, ModuleElement<'a>, VerboseError<&'a str>> {
    tuple((
        tag("target"),
        preceded(space0, tag("datalayout")),
        preceded(space0, char('=')),
        preceded(space0, parse_string_literal),
    ))(source)
    .map(|(i, (_, _, _, datalayout))| (i, ModuleElement::TargetDatalayout(datalayout)))
}

fn parse_target_triple<'a>(
    source: &'a str,
) -> IResult<&'a str, ModuleElement<'a>, VerboseError<&'a str>> {
    tuple((
        tag("target"),
        preceded(space0, tag("triple")),
        preceded(space0, char('=')),
        preceded(space0, parse_string_literal),
    ))(source)
    .map(|(i, (_, _, _, triple))| (i, ModuleElement::TargetTriple(triple)))
}

pub fn parse_module<'a>(mut source: &'a str) -> Result<Module, Err<VerboseError<&'a str>>> {
    let mut module = Module::new();

    loop {
        let (source_, (_, element, _)) = tuple((
            multispace0,
            alt((
                parse_source_filename,
                parse_target_datalayout,
                parse_target_triple,
                // TODO: How do I deal with comments?
                map(
                    preceded(char(';'), terminated(take_until("\n"), char('\n'))),
                    |_| ModuleElement::Comment,
                ),
            )),
            multispace0,
        ))(source)?;

        match element {
            ModuleElement::SourceFilename(name) => {
                module.source_filename = name.to_string();
            }
            ModuleElement::TargetDatalayout(datalayout) => {
                module.target.datalayout = datalayout.to_string();
            }
            ModuleElement::TargetTriple(triple) => {
                module.target.triple = triple.to_string();
            }
            ModuleElement::Comment => {}
        }

        if source_.is_empty() {
            break;
        }
        source = source_
    }

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
