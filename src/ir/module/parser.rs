use super::super::function;
use super::attributes::{parser::parse_attributes, Attribute};
use super::Module;
use crate::ir::util::spaces;
use nom::{
    bytes::complete::{tag, take_until},
    character::complete::{char, digit1},
    combinator::{cut, map},
    error::VerboseError,
    sequence::{preceded, terminated, tuple},
    Err, IResult,
};

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
            module.attributes.insert(id, attrs);
            source = source_;
            continue;
        }

        if let Ok((source_, func)) = function::parse(source, module.types.clone()) {
            module.functions.alloc(func);
            source = source_;
            continue;
        }

        if let Ok((source_, _)) = parse_metadata(source) {
            source = source_;
            continue;
        }
    }

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
            define dso_local i32 @main(i32 %0, i32 %1) #0 {
                ret void
            }
            attributes #0 = { noinline "abcde" = ;fff
            ;ff
                                            "ff" "xxx"}
        "#,
    );
    assert!(result.is_ok());
    let result = result.unwrap();
    assert_eq!(result.source_filename, "c.c");
    assert_eq!(
        result.target.datalayout,
        "e-m:e-p270:32:32-p271:32:32-p272:64:64-i64:64-f80:128-n8:16:32:64-S128"
    );
    assert_eq!(result.target.triple, "x86_64-pc-linux-gnu");
    println!("{:?}", result);
}

#[test]
#[rustfmt::skip]
fn parse_module2() {
    use rustc_hash::FxHashMap;
    let result = parse(include_str!("../../../examples/ret42.ll"));
    assert!(result.is_ok());
    let result = result.unwrap();
    assert_eq!(result.source_filename, "c.c");
    assert_eq!(
        result.target.datalayout,
        "e-m:e-p270:32:32-p271:32:32-p272:64:64-i64:64-f80:128-n8:16:32:64-S128"
    );
    assert_eq!(result.target.triple, "x86_64-pc-linux-gnu");
    let attrs = vec![(
        0u32,
        vec![
            Attribute::NoInline,
            Attribute::NoUnwind,
            Attribute::OptNone,
            Attribute::UWTable, 
            Attribute::StringAttribute {kind: "correctly-rounded-divide-sqrt-fp-math".to_string() , value: "false"                          .to_string()} ,
            Attribute::StringAttribute {kind: "disable-tail-calls"                   .to_string() , value: "false"                          .to_string()} ,
            Attribute::StringAttribute {kind: "frame-pointer"                        .to_string() , value: "all"                            .to_string()} ,
            Attribute::StringAttribute {kind: "less-precise-fpmad"                   .to_string() , value: "false"                          .to_string()} ,
            Attribute::StringAttribute {kind: "min-legal-vector-width"               .to_string() , value: "0"                              .to_string()} ,
            Attribute::StringAttribute {kind: "no-infs-fp-math"                      .to_string() , value: "false"                          .to_string()} ,
            Attribute::StringAttribute {kind: "no-jump-tables"                       .to_string() , value: "false"                          .to_string()} ,
            Attribute::StringAttribute {kind: "no-nans-fp-math"                      .to_string() , value: "false"                          .to_string()} ,
            Attribute::StringAttribute {kind: "no-signed-zeros-fp-math"              .to_string() , value: "false"                          .to_string()} ,
            Attribute::StringAttribute {kind: "no-trapping-math"                     .to_string() , value: "false"                          .to_string()} ,
            Attribute::StringAttribute {kind: "stack-protector-buffer-size"          .to_string() , value: "8"                              .to_string()} ,
            Attribute::StringAttribute {kind: "target-cpu"                           .to_string() , value: "x86-64"                         .to_string()} ,
            Attribute::StringAttribute {kind: "target-features"                      .to_string() , value: "+cx8,+fxsr,+mmx,+sse,+sse2,+x87".to_string()} ,
            Attribute::StringAttribute {kind: "unsafe-fp-math"                       .to_string() , value: "false"                          .to_string()} ,
            Attribute::StringAttribute {kind: "use-soft-float"                       .to_string() , value: "false"                          .to_string()} ,
        ],
    )]
    .into_iter()
    .collect::<FxHashMap<u32, Vec<Attribute>>>();
    for (key1, val1) in &result.attributes {
        assert!(&attrs[key1] == val1)
    }
    println!("{:?}", result);
}
