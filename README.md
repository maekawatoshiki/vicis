# Vicis

[![CI](https://github.com/maekawatoshiki/vicis/workflows/Rust/badge.svg)](https://circleci.com/gh/maekawatoshiki/vicis)
[![codecov](https://codecov.io/gh/maekawatoshiki/vicis/branch/master/graph/badge.svg)](https://codecov.io/gh/maekawatoshiki/vicis)
[![](http://img.shields.io/badge/license-MIT-blue.svg)](./LICENSE)

Manipulate LLVM-IR in Pure Rust

# TODO

- [ ] Implement parser for all LLVM Assembly features 
- [ ] Implement interpreter and code generator for some architectures (WIP x86)
- [ ] Write documents

# Requirements

- clang (>= 10.0.0 recommended) is used for tests

# Example

- Iterate over instructions

```rust
fn main() {
    let asm = r#"
      source_filename = "asm"
      target datalayout = "e-m:e-p270:32:32-p271:32:32-p272:64:64-i64:64-f80:128-n8:16:32:64-S128"     
      target triple = "x86_64-pc-linux-gnu"  

      ; Function Attrs: noinline nounwind optnone uwtable
      define dso_local i32 @main() #0 {
        %1 = alloca i32, align 4
        store i32 42, i32* %1
        ret i32 0
      }

      attributes #0 = { noinline nounwind optnone uwtable }
    "#;

    // Parse the assembly and get a module
    let module = module::parse_assembly(asm).unwrap();

    run_on_module(&module);
}

fn run_on_module(module: &Module) {
    for (_, function) in module.functions() {
        run_on_function(function);
    }
}

fn run_on_function(func: &Function) {
    for block_id in func.layout.block_iter() {
        for inst_id in func.layout.inst_iter(block_id) {
            let inst = func.data.inst_ref(inst_id);
            // Do something with `inst` ....
        }
    }
}
```

- Compile LLVM-IR into machine code

```rust
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
let module = module::parse_assembly(asm).unwrap();

// Compile the module for x86 and get a machine module
let mach_module = compile_module(X86_64, module);

// Display the machine module as assembly
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

```

