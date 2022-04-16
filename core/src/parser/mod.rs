use crate::ir::{function::Function, module::Module, types::Type, value::ConstantValue};

pub mod assembly;
pub mod bitcode;
mod parser;

#[derive(Default)]
pub struct Context {
    module: Module,
    func: Function,
}

type TypedConstant = impl FnOnce(Type) -> ConstantValue;

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
            %p = type i32*
            %a = type [10 x i32*]
            %"aaa!!ああ" = type void
            %s1 = type {i32, i32 }
            %s2 = type <{ i8, i16 }>
            %f = type i32 (i8, %p)
            @a = common global i32 
            ; comment :)
            "#,
        )
        .unwrap();
    println!("{:?}", ctx.module);
}
