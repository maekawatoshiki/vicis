use vicis_ir::{ir::module, pass::transform::mem2reg::Mem2Reg};

#[test]
fn mem2reg_1() {
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
        // println!("{:?}", func);
    }
    insta::assert_debug_snapshot!(module);
}

#[test]
fn mem2reg_2() {
    let ir = r#"
define dso_local i32 @main() {
  %1 = alloca i32, align 4
  %2 = alloca i32, align 4
  %3 = alloca i32, align 4
  %4 = alloca i32, align 4
  %5 = alloca i32, align 4
  store i32 0, i32* %1, align 4
  store i32 1, i32* %2, align 4
  %6 = load i32, i32* %2, align 4
  %7 = add nsw i32 %6, 1
  store i32 %7, i32* %3, align 4
  %8 = load i32, i32* %2, align 4
  %9 = load i32, i32* %3, align 4
  %10 = sub nsw i32 %8, %9
  store i32 %10, i32* %4, align 4
  store i32 3, i32* %2, align 4
  %11 = load i32, i32* %2, align 4
  store i32 %11, i32* %5, align 4
  %12 = load i32, i32* %3, align 4
  %13 = load i32, i32* %5, align 4
  %14 = add nsw i32 %12, %13
  ret i32 %14
}"#;
    let mut module = module::parse_assembly(ir).expect("failed to parse ir");
    for (_, func) in module.functions_mut() {
        Mem2Reg::new(func).run();
        // println!("{:?}", func);
    }
    insta::assert_debug_snapshot!(module);
}

#[test]
fn mem2reg_3() {
    let ir = r#"
define dso_local i32 @main() {
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
}"#;
    let mut module = module::parse_assembly(ir).expect("failed to parse ir");
    for (_, func) in module.functions_mut() {
        Mem2Reg::new(func).run();
        // println!("{:?}", func);
    }
    insta::assert_debug_snapshot!(module);
}

#[test]
fn mem2reg_4() {
    let ir = r#"
define dso_local i32 @main() {
  %1 = alloca i32, align 4
  %2 = alloca i32, align 4
  %3 = alloca i32, align 4
  %4 = alloca i32, align 4
  %5 = alloca i32, align 4
  store i32 0, i32* %1, align 4
  store i32 1, i32* %2, align 4
  store i32 1, i32* %3, align 4
  store i32 0, i32* %4, align 4
  br label %6

6:                                                ; preds = %15, %0
  %7 = load i32, i32* %4, align 4
  %8 = icmp slt i32 %7, 9
  br i1 %8, label %9, label %18

9:                                                ; preds = %6
  %10 = load i32, i32* %2, align 4
  store i32 %10, i32* %5, align 4
  %11 = load i32, i32* %3, align 4
  store i32 %11, i32* %2, align 4
  %12 = load i32, i32* %5, align 4
  %13 = load i32, i32* %3, align 4
  %14 = add nsw i32 %12, %13
  store i32 %14, i32* %3, align 4
  br label %15

15:                                               ; preds = %9
  %16 = load i32, i32* %4, align 4
  %17 = add nsw i32 %16, 1
  store i32 %17, i32* %4, align 4
  br label %6

18:                                               ; preds = %6
  %19 = load i32, i32* %3, align 4
  ret i32 %19
}
"#;
    let mut module = module::parse_assembly(ir).expect("failed to parse ir");
    for (_, func) in module.functions_mut() {
        Mem2Reg::new(func).run();
        // println!("{:?}", func);
    }
    insta::assert_debug_snapshot!(module);
}
