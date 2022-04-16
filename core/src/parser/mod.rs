pub mod assembly;
pub mod bitcode;
mod parser;

#[test]
fn parsertest() {
    let mut module = crate::ir::module::Module::default();
    parser::ModuleParser::new()
        .parse(&mut module, r#"source_filename = "a.c""#)
        .unwrap();
    println!("{:?}", module);
}
