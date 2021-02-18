use vicis::{
    // codegen::{isa::x86_64::X86_64, lower::compile_module},
    // exec::{generic_value::GenericValue, interpreter::Interpreter},
    ir::module,
    pass::analysis::dom_tree::DominatorTree,
};

#[test]
fn dom1() {
    let src = r#"
    source_filename = "c.c"                                                                            
target datalayout = "e-m:e-p270:32:32-p271:32:32-p272:64:64-i64:64-f80:128-n8:16:32:64-S128"       
target triple = "x86_64-pc-linux-gnu"                                                              
         
define dso_local i32 @main() {                                            
  %1 = alloca i32, align 4                                                   
  %2 = alloca i32, align 4                                                   
  %3 = alloca i32, align 4                                                   
  store i32 0, i32* %1, align 4                                              
  store i32 1, i32* %2, align 4                                              
  %4 = load i32, i32* %2, align 4                                            
  %5 = icmp slt i32 %4, 2                                                    
  br i1 %5, label %6, label %11                                              
                                                                             
6:                                                ; preds = %0               
  store i32 10, i32* %3, align 4                                             
  %7 = load i32, i32* %2, align 4                                            
  %8 = icmp slt i32 %7, 5                                                    
  br i1 %8, label %9, label %10                                              
                                                                             
9:                                                ; preds = %6               
  store i32 30, i32* %3, align 4                                             
  br label %10                                                               
                                                                             
10:                                               ; preds = %9, %6           
  br label %12                                                               
                                                                             
11:                                               ; preds = %0               
  store i32 20, i32* %3, align 4                                             
  br label %12                                                               
                                                                             
12:                                               ; preds = %11, %10         
  %13 = load i32, i32* %3, align 4                                           
  ret i32 %13                                                                
}                                                                            

        "#;

    let module = module::parse_assembly(src).unwrap();

    println!("{:?}", module);

    for (_, func) in module.functions() {
        let dom_tree = DominatorTree::new(func);
        println!("{:#?}", dom_tree);
    }
}
