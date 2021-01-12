use vicis::{
    codegen::{
        lower::convert_module,
        target::x86_64::{calling_conv::SystemV, X86_64},
    },
    // exec::{generic_value::GenericValue, interpreter::Interpreter},
    ir::module,
};

#[test]
fn gen() {
    let asm = r#"
source_filename = "asm.c"                                                                          
target datalayout = "e-m:e-p270:32:32-p271:32:32-p272:64:64-i64:64-f80:128-n8:16:32:64-S128"     
target triple = "x86_64-pc-linux-gnu"                                                            

; Function Attrs: noinline nounwind optnone uwtable                                              
define dso_local i32 @main() #0 {                                                                
  %a = alloca i32, align 4
  store i32 2, i32* %a
  %b = load i32, i32* %a
  %c = add i32 %b, 1 ; 3
  %d = add i32 %b, 2 ; 4
  %e = add i32 %c, %d ; 7
  ret i32 %e
}                                                                                                

attributes #0 = { noinline nounwind optnone uwtable }
"#;
    let module = module::parse_assembly(asm).unwrap();
    // let main = module.find_function_by_name("main").unwrap();
    let mach_module = convert_module(X86_64::new(SystemV), module);
    assert_eq!(
        format!("{}", mach_module),
        "  .text
  .intel_syntax noprefix
  .globl main
main:
.LBL0:
  push rbp
  mov rbp, rsp
  sub rsp, 16
  mov dword ptr [rbp-4], 2
  mov eax, dword ptr [rbp-4]
  mov ecx, eax
  add ecx, 1
  add eax, 2
  add ecx, eax
  mov eax, ecx
  add rsp, 16
  pop rbp
  ret 
"
    );
    println!("{}", mach_module);
}

#[test]
fn br() {
    let asm = r#"
source_filename = "asm.c"                                                                          
target datalayout = "e-m:e-p270:32:32-p271:32:32-p272:64:64-i64:64-f80:128-n8:16:32:64-S128"     
target triple = "x86_64-pc-linux-gnu"                                                            

; Function Attrs: noinline nounwind optnone uwtable                                              
define dso_local i32 @main() #0 {                                                                
  %a = alloca i32, align 4
  store i32 2, i32* %a
  br label %bb
bb:
  %b = load i32, i32* %a
  ret i32 %b
}                                                                                                

attributes #0 = { noinline nounwind optnone uwtable }
"#;
    let module = module::parse_assembly(asm).unwrap();
    // let main = module.find_function_by_name("main").unwrap();
    let mach_module = convert_module(X86_64::new(SystemV), module);
    assert_eq!(
        format!("{}", mach_module),
        "  .text
  .intel_syntax noprefix
  .globl main
main:
.LBL0:
  push rbp
  mov rbp, rsp
  sub rsp, 16
  mov dword ptr [rbp-4], 2
  jmp .LBL1
.LBL1:
  mov eax, dword ptr [rbp-4]
  add rsp, 16
  pop rbp
  ret 
"
    );
    println!("{}", mach_module);
}

#[test]
fn condbr() {
    let asm = r#"
source_filename = "asm.c"                                                                          
target datalayout = "e-m:e-p270:32:32-p271:32:32-p272:64:64-i64:64-f80:128-n8:16:32:64-S128"     
target triple = "x86_64-pc-linux-gnu"                                                            

; Function Attrs: noinline nounwind optnone uwtable                                              
define dso_local i32 @main() #0 {                                                                
  %a = alloca i32, align 4
  store i32 2, i32* %a
  %b = load i32, i32* %a
  %c = icmp eq i32 %b, 2
  br i1 %c, label %b1, label %b2
b1:
  ret i32 1
b2:
  ret i32 2
}                                                                                                

attributes #0 = { noinline nounwind optnone uwtable }
"#;
    let module = module::parse_assembly(asm).unwrap();
    // let main = module.find_function_by_name("main").unwrap();
    let mach_module = convert_module(X86_64::new(SystemV), module);
    assert_eq!(
        format!("{}", mach_module),
        "  .text
  .intel_syntax noprefix
  .globl main
main:
.LBL0:
  push rbp
  mov rbp, rsp
  sub rsp, 16
  mov dword ptr [rbp-4], 2
  mov eax, dword ptr [rbp-4]
  cmp eax, 2
  je .LBL1
  jmp .LBL2
.LBL1:
  mov eax, 1
  add rsp, 16
  pop rbp
  ret 
.LBL2:
  mov eax, 2
  add rsp, 16
  pop rbp
  ret 
"
    );
    println!("{}", mach_module);
}

#[test]
fn sum() {
    let asm = r#"
; ModuleID = 'c.c'
source_filename = "c.c"
target datalayout = "e-m:e-p270:32:32-p271:32:32-p272:64:64-i64:64-f80:128-n8:16:32:64-S128"
target triple = "x86_64-pc-linux-gnu"

; Function Attrs: noinline nounwind optnone uwtable
define dso_local i32 @main() #0 {
  %1 = alloca i32, align 4
  %2 = alloca i32, align 4
  %3 = alloca i32, align 4
  store i32 0, i32* %1, align 4
  store i32 0, i32* %2, align 4
  store i32 1, i32* %3, align 4
  br label %4

4:                                                ; preds = %11, %0
  %5 = load i32, i32* %3, align 4
  %6 = icmp sle i32 %5, 10
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

attributes #0 = { noinline nounwind optnone uwtable  }

!llvm.module.flags = !{!0}
!llvm.ident = !{!1}

!0 = !{i32 1, !"wchar_size", i32 4}
!1 = !{!"clang version 10.0.0-4ubuntu1 "}
"#;
    let module = module::parse_assembly(asm).unwrap();
    // let main = module.find_function_by_name("main").unwrap();
    let mach_module = convert_module(X86_64::new(SystemV), module);
    println!("{}", mach_module);
}

#[test]
fn phi() {
    let asm = r#"
; ModuleID = 'c.ll'
source_filename = "c.c"
target datalayout = "e-m:e-p270:32:32-p271:32:32-p272:64:64-i64:64-f80:128-n8:16:32:64-S128"
target triple = "x86_64-pc-linux-gnu"

; Function Attrs: noinline nounwind uwtable
define dso_local i32 @main() #0 {
  %0 = alloca i32
  store i32 1, i32* %0
  %1 = load i32, i32* %0
  %2 = icmp eq i32 %1, 0
  br i1 %2, label %3, label %4

3:                                                ; preds = %1
  br label %5

4:                                                ; preds = %1
  br label %5

5:                                                ; preds = %4, %3
  %.0 = phi i32 [ 1, %3 ], [ 2, %4 ]
  ret i32 %.0
}

attributes #0 = { noinline nounwind uwtable }

!llvm.module.flags = !{!0}
!llvm.ident = !{!1}

!0 = !{i32 1, !"wchar_size", i32 4}
!1 = !{!"clang version 10.0.0-4ubuntu1 "}
    "#;
    let module = module::parse_assembly(asm).unwrap();
    println!("{:?}", module);
    // let main = module.find_function_by_name("main").unwrap();
    let mach_module = convert_module(X86_64::new(SystemV), module);
    println!("{}", mach_module);
}

#[test]
fn phi2() {
    let asm = r#"
; ModuleID = 'c.ll'
source_filename = "c.c"
target datalayout = "e-m:e-p270:32:32-p271:32:32-p272:64:64-i64:64-f80:128-n8:16:32:64-S128"
target triple = "x86_64-pc-linux-gnu"

; Function Attrs: noinline nounwind uwtable
define dso_local i32 @main() #0 {
  br label %1

1:                                                ; preds = %5, %0
  %.01 = phi i32 [ 0, %0 ], [ %4, %5 ]
  %.0 = phi i32 [ 1, %0 ], [ %6, %5 ]
  %2 = icmp sle i32 %.0, 10
  br i1 %2, label %3, label %7

3:                                                ; preds = %1
  %4 = add nsw i32 %.01, %.0
  br label %5

5:                                                ; preds = %3
  %6 = add nsw i32 %.0, 1
  br label %1

7:                                                ; preds = %1
  ret i32 %.01
}

attributes #0 = { noinline nounwind uwtable }

!llvm.module.flags = !{!0}
!llvm.ident = !{!1}

!0 = !{i32 1, !"wchar_size", i32 4}
!1 = !{!"clang version 10.0.0-4ubuntu1 "}
    "#;
    let module = module::parse_assembly(asm).unwrap();
    println!("{:?}", module);
    // let main = module.find_function_by_name("main").unwrap();
    let mach_module = convert_module(X86_64::new(SystemV), module);
    println!("{}", mach_module);
}
