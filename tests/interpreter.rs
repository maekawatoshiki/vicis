use vicis::{
    exec::{generic_value::GenericValue, interpreter},
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
    assert_eq!(
        interpreter::run_function(&module, main).unwrap(),
        GenericValue::Int32(0)
    );
}

#[test]
fn exec2() {
    let asm = r#"
; ModuleID = 'a.c'
source_filename = "a.c"
target datalayout = "e-m:e-p270:32:32-p271:32:32-p272:64:64-i64:64-f80:128-n8:16:32:64-S128"
target triple = "x86_64-pc-linux-gnu"

; Function Attrs: noinline nounwind optnone uwtable
define dso_local i32 @main() #0 {
  %1 = alloca i32, align 4
  %2 = alloca i32, align 4
  %3 = alloca i32, align 4
  store i32 0, i32* %1, align 4
  store i32 0, i32* %2, align 4
  store i32 0, i32* %3, align 4
  br label %4

4:                                                ; preds = %11, %0
  %5 = load i32, i32* %3, align 4
  %6 = icmp slt i32 %5, 11
  br i1 %6, label %7, label %14

7:                                                ; preds = %4
  %8 = load i32, i32* %3, align 4
  %9 = load i32, i32* %2, align 4
  %10 = add nsw i32 %9, %8
  store i32 %10, i32* %2, align 4
  br label %11

11:                                               ; preds = %7
  %12 = load i32, i32* %3, align 4
  %13 = add nsw i32 %12, 1
  store i32 %13, i32* %3, align 4
  br label %4

14:                                               ; preds = %4
  %15 = load i32, i32* %2, align 4
  ret i32 %15
}

attributes #0 = { noinline nounwind optnone uwtable "correctly-rounded-divide-sqrt-fp-math"="false" "disable-tail-calls"="false" "frame-pointer"="all" "less-precise-fpmad"="false" "min-legal-vector-width"="0" "no-infs-fp-math"="false" "no-jump-tables"="false" "no-nans-fp-math"="false" "no-signed-zeros-fp-math"="false" "no-trapping-math"="false" "stack-protector-buffer-size"="8" "target-cpu"="x86-64" "target-features"="+cx8,+fxsr,+mmx,+sse,+sse2,+x87" "unsafe-fp-math"="false" "use-soft-float"="false" }

!llvm.module.flags = !{!0}
!llvm.ident = !{!1}

!0 = !{i32 1, !"wchar_size", i32 4}
!1 = !{!"clang version 10.0.0-4ubuntu1 "}
    "#;
    let module = module::parse_assembly(asm).unwrap();
    let main = module.find_function_by_name("main").unwrap();
    assert_eq!(
        interpreter::run_function(&module, main).unwrap(),
        GenericValue::Int32(55)
    );
}
