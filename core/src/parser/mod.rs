use crate::ir::{function::Function, module::Module};

pub mod assembly;
pub mod bitcode;
mod parser;

#[derive(Default)]
pub struct Context {
    module: Module,
    func: Function,
}

#[test]
fn parsertest() {
    let mut ctx = Context::default();
    parser::ModuleParser::new()
        .parse(
            &mut ctx,
            r#"
            source_filename = "a.c"
            target datalayout = "xxxxx"
            target triple = "xxxxx"
            attributes #0 = { alwaysinline nounwind }
            %aaa_ = type i32
            %"aaa!!ああ" = type void
            "#,
        )
        .unwrap();
    println!("{:?}", ctx.module);
}
