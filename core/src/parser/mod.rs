pub mod assembly;
pub mod bitcode;
mod parser;

#[test]
fn parsertest() {
    let mut module = crate::ir::module::Module::default();
    parser::ModuleParser::new()
        .parse(
            &mut module,
            r#"
            source_filename = "a.c"
            target datalayout = "xxxxx"
            attributes #0 = { alwaysinline  }
            "#,
        )
        .unwrap();
    println!("{:?}", module);
}
