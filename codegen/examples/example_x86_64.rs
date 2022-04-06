use vicis_codegen::{self, isa::x86_64::X86_64, lower::compile_module};
use vicis_core::ir::module::Module;

fn main() {
    // LLVM Assembly
    let asm = r#"
  source_filename = "asm"
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

    // Parse the assembly and get a module
    let module = Module::try_from(asm).expect("failed to parse LLVM IR");

    // Compile the module for x86 and get a machine module
    let isa = X86_64::default();
    let mach_module = compile_module(&isa, &module).expect("failed to compile");

    // Display the machine module as assembly
    assert_eq!(
        format!("{}", mach_module.display_asm()),
        "  .text
  .intel_syntax noprefix
  .text
  .globl main
main:
.LBL0_0:
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
}
