use vicis::{
    exec::{generic_value::GenericValue, interpreter::Interpreter},
    ir::module,
};

#[test]
fn exec() {
    let asm = r#"
source_filename = "asm.c"                                                                          
target datalayout = "e-m:e-p270:32:32-p271:32:32-p272:64:64-i64:64-f80:128-n8:16:32:64-S128"     
target triple = "x86_64-pc-linux-gnu"                                                            
                                                                                                 
; Function Attrs: noinline nounwind optnone uwtable                                              
define dso_local i32 @main() #0 {                                                                
  %1 = alloca i32, align 4                                                                       
  %2 = alloca i32, align 4                                                                       
  store i32 0, i32* %1, align 4                                                                  
  store i32 5, i32* %2, align 4                                                                 
  %3 = load i32, i32* %2, align 4                                                                
  %4 = add nsw i32 %3, 2
  %5 = sub nsw i32 %4, 7
  ret i32 %5 ; must be 0 
}                                                                                                
                                                                                                 
attributes #0 = { noinline nounwind optnone uwtable }
"#;
    let module = module::parse_assembly(asm).unwrap();
    let main = module.find_function_by_name("main").unwrap();
    let mut interpreter = Interpreter::new(&module);
    assert_eq!(
        interpreter.run_function(main).unwrap(),
        GenericValue::Int32(0)
    );
}
