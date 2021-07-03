use vicis_ir::{ir::module, pass::transform::mem2reg::Mem2Reg};

#[test]
fn mem2reg() {
    let ir = r#"
define dso_local i32 @main() {
  %1 = alloca i32, align 4
  %2 = alloca i32, align 4
  store i32 0, i32* %1, align 4
  store i32 234, i32* %2, align 4
  %3 = load i32, i32* %2, align 4
  %4 = add nsw i32 %3, 123
  ret i32 %4
}"#;
    let mut module = module::parse_assembly(ir).expect("failed to parse ir");
    for (_, func) in module.functions_mut() {
        Mem2Reg::new(func).run();
        println!("{:?}", func);
    }
}
