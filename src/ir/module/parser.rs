use super::Module;
use nom::{
    branch::alt,
    bytes::complete::{tag, take_until},
    character::complete::{char, multispace0},
    combinator::cut,
    error::VerboseError,
    sequence::{preceded, terminated, tuple},
    Err, IResult,
};

enum ModuleElement<'a> {
    SourceFilename(&'a str),
}

fn parse_source_filename<'a>(
    source: &'a str,
) -> IResult<&'a str, ModuleElement<'a>, VerboseError<&'a str>> {
    tuple((
        tag("source_filename"),
        preceded(multispace0, char('=')),
        preceded(
            multispace0,
            preceded(char('\"'), cut(terminated(take_until("\""), char('\"')))),
        ),
        preceded(multispace0, char(';')),
    ))(source)
    .map(|(i, (_, _, name, _))| (i, ModuleElement::SourceFilename(name)))
}

pub fn parse_module<'a>(mut source: &'a str) -> Result<Module, Err<VerboseError<&'a str>>> {
    let mut module = Module::new();

    loop {
        let (source_, (_, element, _)) = tuple((
            multispace0,
            alt((
                parse_source_filename,
                parse_source_filename,
                parse_source_filename,
            )),
            multispace0,
        ))(source)?;

        match element {
            ModuleElement::SourceFilename(name) => {
                module.source_filename = name.to_string();
            }
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
            source_filename = "c.c";
        "#,
    );
    assert!(result.is_ok());
    let result = result.unwrap();
    assert_eq!(result.source_filename, "c.c")
}
