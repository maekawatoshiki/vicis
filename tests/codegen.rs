use vicis::{
    codegen::{lower::convert_module, target::x86_64::X86_64},
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
    let mach_module = convert_module(X86_64, module);
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
    let mach_module = convert_module(X86_64, module);
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
    let mach_module = convert_module(X86_64, module);
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
    let mach_module = convert_module(X86_64, module);
    assert_eq!(
        format!("{}", mach_module),
        r#"  .text
  .intel_syntax noprefix
  .globl main
main:
.LBL0:
  push rbp
  mov rbp, rsp
  sub rsp, 16
  mov dword ptr [rbp-12], 0
  mov dword ptr [rbp-4], 0
  mov dword ptr [rbp-8], 1
  jmp .LBL1
.LBL1:
  mov eax, dword ptr [rbp-8]
  cmp eax, 10
  jle .LBL2
  jmp .LBL4
.LBL2:
  mov eax, dword ptr [rbp-8]
  mov ecx, dword ptr [rbp-4]
  add ecx, eax
  mov dword ptr [rbp-4], ecx
  jmp .LBL3
.LBL3:
  mov eax, dword ptr [rbp-8]
  add eax, 1
  mov dword ptr [rbp-8], eax
  jmp .LBL1
.LBL4:
  mov eax, dword ptr [rbp-4]
  add rsp, 16
  pop rbp
  ret 
"#
    );
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
    let mach_module = convert_module(X86_64, module);
    assert_eq!(
        format!("{}", mach_module),
        r#"  .text
  .intel_syntax noprefix
  .globl main
main:
.LBL0:
  push rbp
  mov rbp, rsp
  sub rsp, 16
  mov dword ptr [rbp-4], 1
  mov eax, dword ptr [rbp-4]
  cmp eax, 0
  je .LBL1
  jmp .LBL2
.LBL1:
  mov eax, 1
  jmp .LBL3
.LBL2:
  mov eax, 2
  jmp .LBL3
.LBL3:
  add rsp, 16
  pop rbp
  ret 
"#
    );
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
    let mach_module = convert_module(X86_64, module);
    assert_eq!(
        format!("{}", mach_module),
        r#"  .text
  .intel_syntax noprefix
  .globl main
main:
.LBL0:
  push rbp
  mov rbp, rsp
  mov eax, 0
  mov ecx, 1
  jmp .LBL1
.LBL1:
  cmp ecx, 10
  jle .LBL2
  jmp .LBL4
.LBL2:
  add eax, ecx
  jmp .LBL3
.LBL3:
  add ecx, 1
  jmp .LBL1
.LBL4:
  pop rbp
  ret 
"#
    );
}

#[test]
fn call1() {
    let asm = r#"
define dso_local i32 @f() #0 {
  ret i32 1
}

; Function Attrs: noinline nounwind optnone uwtable
define dso_local i32 @main() #0 {
  %1 = alloca i32, align 4
  store i32 0, i32* %1, align 4
  %2 = call i32 @f()
  ret i32 %2
}
    "#;
    let module = module::parse_assembly(asm).unwrap();
    println!("{:?}", module);
    let mach_module = convert_module(X86_64, module);
    println!("{}", format!("{}", mach_module));
    assert_eq!(
        format!("{}", mach_module),
        r#"  .text
  .intel_syntax noprefix
  .globl f
f:
.LBL0:
  push rbp
  mov rbp, rsp
  mov eax, 1
  pop rbp
  ret 
  .globl main
main:
.LBL0:
  push rbp
  mov rbp, rsp
  sub rsp, 16
  mov dword ptr [rbp-4], 0
  call f
  add rsp, 16
  pop rbp
  ret 
"#
    );
}

#[test]
fn call2() {
    let asm = r#"
define dso_local i32 @f(i32 %a) #0 {
  ret i32 %a
}

; Function Attrs: noinline nounwind optnone uwtable
define dso_local i32 @main() #0 {
  %1 = call i32 @f(i32 1)
  ret i32 %1
}
    "#;
    let module = module::parse_assembly(asm).unwrap();
    println!("{:?}", module);
    let mach_module = convert_module(X86_64, module);
    assert_eq!(
        format!("{}", mach_module),
        r#"  .text
  .intel_syntax noprefix
  .globl f
f:
.LBL0:
  push rbp
  mov rbp, rsp
  mov eax, edi
  pop rbp
  ret 
  .globl main
main:
.LBL0:
  push rbp
  mov rbp, rsp
  mov edi, 1
  call f
  pop rbp
  ret 
"#
    );
}

#[test]
fn ary1() {
    let asm = r#"
  ; ModuleID = 'c.c'
source_filename = "c.c"
target datalayout = "e-m:e-p270:32:32-p271:32:32-p272:64:64-i64:64-f80:128-n8:16:32:64-S128"
target triple = "x86_64-pc-linux-gnu"

; Function Attrs: noinline nounwind optnone uwtable
define dso_local i32 @main() #0 {
  %1 = alloca i32, align 4
  %2 = alloca [4 x i32], align 16
  store i32 0, i32* %1, align 4
  %3 = getelementptr inbounds [4 x i32], [4 x i32]* %2, i64 0, i64 0
  store i32 0, i32* %3, align 16
  %4 = getelementptr inbounds [4 x i32], [4 x i32]* %2, i64 0, i64 1
  store i32 1, i32* %4, align 4
  %5 = getelementptr inbounds [4 x i32], [4 x i32]* %2, i64 0, i64 2
  store i32 2, i32* %5, align 8
  %6 = getelementptr inbounds [4 x i32], [4 x i32]* %2, i64 0, i64 3
  store i32 3, i32* %6, align 4
  ret i32 0
}

attributes #0 = { noinline nounwind optnone uwtable "correctly-rounded-divide-sqrt-fp-math"="false" "disable-tail-calls"="false" "frame-pointer"="all" "less-precise-fpmad"="false" "min-legal-vector-width"="0" "no-infs-fp-math"="false" "no-jump-tables"="false" "no-nans-fp-math"="false" "no-signed-zeros-fp-math"="false" "no-trapping-math"="false" "stack-protector-buffer-size"="8" "target-cpu"="x86-64" "target-features"="+cx8,+fxsr,+mmx,+sse,+sse2,+x87" "unsafe-fp-math"="false" "use-soft-float"="false" }

!llvm.module.flags = !{!0}
!llvm.ident = !{!1}

!0 = !{i32 1, !"wchar_size", i32 4}
!1 = !{!"clang version 10.0.0-4ubuntu1 "}
  "#;
    let module = module::parse_assembly(asm).unwrap();
    println!("{:?}", module);
    let mach_module = convert_module(X86_64, module);
    assert_eq!(
        format!("{}", mach_module),
        r#"  .text
  .intel_syntax noprefix
  .globl main
main:
.LBL0:
  push rbp
  mov rbp, rsp
  sub rsp, 32
  mov dword ptr [rbp-20], 0
  mov dword ptr [rbp-16], 0
  mov dword ptr [rbp-12], 1
  mov dword ptr [rbp-8], 2
  mov dword ptr [rbp-4], 3
  mov eax, 0
  add rsp, 32
  pop rbp
  ret 
"#
    );
}

// #[test]
// fn run_file() {
//     let path = "";
//     let asm = include_str!(path);
//     let module = module::parse_assembly(asm).unwrap();
//     println!("{:?}", module);
//     // let main = module.find_function_by_name("main").unwrap();
//     let mach_module = convert_module(X86_64, module);
//     println!("{}", mach_module);
// }
