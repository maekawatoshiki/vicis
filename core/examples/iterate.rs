use vicis_core::ir::{function::Function, module::Module};

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
    let module = Module::try_from(asm).expect("failed to parse LLVM IR");

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
            // Do something with the instruction here...
            println!("instruction: {}", inst.display(&func.data, &func.types));
        }
    }
}
