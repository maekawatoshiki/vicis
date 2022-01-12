use vicis_core::ir::module;
use vicis_interpreter::{generic_value::GenericValue, interpreter};

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
    assert_eq!(run(asm, vec![]), GenericValue::Int32(0));
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
    assert_eq!(run(asm, vec![]), GenericValue::Int32(55));
}

#[test]
fn exec3() {
    let asm = r#"
; ModuleID = 'c.c'
source_filename = "c.c"
target datalayout = "e-m:e-p270:32:32-p271:32:32-p272:64:64-i64:64-f80:128-n8:16:32:64-S128"
target triple = "x86_64-pc-linux-gnu"

; Function Attrs: noinline nounwind optnone uwtable
define dso_local i32 @f() #0 {
  ret i32 123
}

; Function Attrs: noinline nounwind optnone uwtable
define dso_local i32 @main() #0 {
  %1 = alloca i32, align 4
  store i32 0, i32* %1, align 4
  %2 = call i32 @f()
  ret i32 %2
}

attributes #0 = { noinline nounwind optnone uwtable "correctly-rounded-divide-sqrt-fp-math"="false" "disable-tail-calls"="false" "frame-pointer"="all" "less-precise-fpmad"="false" "min-legal-vector-width"="0" "no-infs-fp-math"="false" "no-jump-tables"="false" "no-nans-fp-math"="false" "no-signed-zeros-fp-math"="false" "no-trapping-math"="false" "stack-protector-buffer-size"="8" "target-cpu"="x86-64" "target-features"="+cx8,+fxsr,+mmx,+sse,+sse2,+x87" "unsafe-fp-math"="false" "use-soft-float"="false" }

!llvm.module.flags = !{!0}
!llvm.ident = !{!1}

!0 = !{i32 1, !"wchar_size", i32 4}
!1 = !{!"clang version 10.0.0-4ubuntu1 "}
    "#;
    assert_eq!(run(asm, vec![]), GenericValue::Int32(123));
}

#[test]
fn exec4() {
    let asm = r#"
; ModuleID = 'c.c'
source_filename = "c.c"
target datalayout = "e-m:e-p270:32:32-p271:32:32-p272:64:64-i64:64-f80:128-n8:16:32:64-S128"
target triple = "x86_64-pc-linux-gnu"

; Function Attrs: noinline nounwind optnone uwtable
define dso_local i32 @f(i32 %0) #0 {
  %2 = alloca i32, align 4
  store i32 %0, i32* %2, align 4
  %3 = load i32, i32* %2, align 4
  %4 = add nsw i32 10, %3
  ret i32 %4
}

; Function Attrs: noinline nounwind optnone uwtable
define dso_local i32 @main() #0 {
  %1 = alloca i32, align 4
  store i32 0, i32* %1, align 4
  %2 = call i32 @f(i32 32)
  ret i32 %2
}

attributes #0 = { noinline nounwind optnone uwtable "correctly-rounded-divide-sqrt-fp-math"="false" "disable-tail-calls"="false" "frame-pointer"="all" "less-precise-fpmad"="false" "min-legal-vector-width"="0" "no-infs-fp-math"="false" "no-jump-tables"="false" "no-nans-fp-math"="false" "no-signed-zeros-fp-math"="false" "no-trapping-math"="false" "stack-protector-buffer-size"="8" "target-cpu"="x86-64" "target-features"="+cx8,+fxsr,+mmx,+sse,+sse2,+x87" "unsafe-fp-math"="false" "use-soft-float"="false" }

!llvm.module.flags = !{!0}
!llvm.ident = !{!1}

!0 = !{i32 1, !"wchar_size", i32 4}
!1 = !{!"clang version 10.0.0-4ubuntu1 "}
    "#;
    assert_eq!(run(asm, vec![]), GenericValue::Int32(42));
}

#[test]
fn exec5() {
    let asm = r#"
; ModuleID = 'c.c'
source_filename = "c.c"
target datalayout = "e-m:e-p270:32:32-p271:32:32-p272:64:64-i64:64-f80:128-n8:16:32:64-S128"
target triple = "x86_64-pc-linux-gnu"

; Function Attrs: noinline nounwind optnone uwtable
define dso_local i32 @f(i32 %0) #0 {
  %2 = alloca i32, align 4
  %3 = alloca i32, align 4
  store i32 %0, i32* %3, align 4
  %4 = load i32, i32* %3, align 4
  %5 = icmp slt i32 %4, 2
  br i1 %5, label %6, label %7

6:                                                ; preds = %1
  store i32 1, i32* %2, align 4
  br label %15

7:                                                ; preds = %1
  %8 = load i32, i32* %3, align 4
  %9 = sub nsw i32 %8, 1
  %10 = call i32 @f(i32 %9)
  %11 = load i32, i32* %3, align 4
  %12 = sub nsw i32 %11, 2
  %13 = call i32 @f(i32 %12)
  %14 = add nsw i32 %10, %13
  store i32 %14, i32* %2, align 4
  br label %15

15:                                               ; preds = %7, %6
  %16 = load i32, i32* %2, align 4
  ret i32 %16
}

; Function Attrs: noinline nounwind optnone uwtable
define dso_local i32 @main() #0 {
  %1 = alloca i32, align 4
  store i32 0, i32* %1, align 4
  %2 = call i32 @f(i32 10)
  ret i32 %2
}

attributes #0 = { noinline nounwind optnone uwtable "correctly-rounded-divide-sqrt-fp-math"="false" "disable-tail-calls"="false" "frame-pointer"="all" "less-precise-fpmad"="false" "min-legal-vector-width"="0" "no-infs-fp-math"="false" "no-jump-tables"="false" "no-nans-fp-math"="false" "no-signed-zeros-fp-math"="false" "no-trapping-math"="false" "stack-protector-buffer-size"="8" "target-cpu"="x86-64" "target-features"="+cx8,+fxsr,+mmx,+sse,+sse2,+x87" "unsafe-fp-math"="false" "use-soft-float"="false" }

!llvm.module.flags = !{!0}
!llvm.ident = !{!1}

!0 = !{i32 1, !"wchar_size", i32 4}
!1 = !{!"clang version 10.0.0-4ubuntu1 "}
    "#;
    assert_eq!(run(asm, vec![]), GenericValue::Int32(89));
}

#[test]
fn exec6() {
    let asm = r#"
; ModuleID = 'c.c'
source_filename = "c.c"
target datalayout = "e-m:e-p270:32:32-p271:32:32-p272:64:64-i64:64-f80:128-n8:16:32:64-S128"
target triple = "x86_64-pc-linux-gnu"

; Function Attrs: noinline nounwind optnone uwtable
define dso_local i32 @main() #0 {
  %1 = alloca i32, align 4
  %2 = alloca [2 x i32], align 4
  store i32 0, i32* %1, align 4
  %3 = getelementptr inbounds [2 x i32], [2 x i32]* %2, i64 0, i64 0
  store i32 1, i32* %3, align 4
  %4 = getelementptr inbounds [2 x i32], [2 x i32]* %2, i64 0, i64 1
  store i32 2, i32* %4, align 4
  %5 = getelementptr inbounds [2 x i32], [2 x i32]* %2, i64 0, i64 0
  %6 = load i32, i32* %5, align 4
  %7 = getelementptr inbounds [2 x i32], [2 x i32]* %2, i64 0, i64 1
  %8 = load i32, i32* %7, align 4
  %9 = add nsw i32 %6, %8
  ret i32 %9
}

attributes #0 = { noinline nounwind optnone uwtable "correctly-rounded-divide-sqrt-fp-math"="false" "disable-tail-calls"="false" "frame-pointer"="all" "less-precise-fpmad"="false" "min-legal-vector-width"="0" "no-infs-fp-math"="false" "no-jump-tables"="false" "no-nans-fp-math"="false" "no-signed-zeros-fp-math"="false" "no-trapping-math"="false" "stack-protector-buffer-size"="8" "target-cpu"="x86-64" "target-features"="+cx8,+fxsr,+mmx,+sse,+sse2,+x87" "unsafe-fp-math"="false" "use-soft-float"="false" }

!llvm.module.flags = !{!0}
!llvm.ident = !{!1}

!0 = !{i32 1, !"wchar_size", i32 4}
!1 = !{!"clang version 10.0.0-4ubuntu1 "}
    "#;
    assert_eq!(run(asm, vec![]), GenericValue::Int32(3));
}

#[test]
fn exec7() {
    let asm = r#"
; ModuleID = 'c.c'
source_filename = "c.c"
target datalayout = "e-m:e-p270:32:32-p271:32:32-p272:64:64-i64:64-f80:128-n8:16:32:64-S128"
target triple = "x86_64-pc-linux-gnu"

; Function Attrs: noinline nounwind optnone uwtable
define dso_local i32 @main() #0 {
  %1 = alloca i32, align 4
  %2 = alloca [10 x i32], align 16
  %3 = alloca i32, align 4
  %4 = alloca i32, align 4
  %5 = alloca i32, align 4
  store i32 0, i32* %1, align 4
  store i32 0, i32* %3, align 4
  br label %6

6:                                                ; preds = %15, %0
  %7 = load i32, i32* %3, align 4
  %8 = icmp slt i32 %7, 10
  br i1 %8, label %9, label %18

9:                                                ; preds = %6
  %10 = load i32, i32* %3, align 4
  %11 = add nsw i32 %10, 1
  %12 = load i32, i32* %3, align 4
  %13 = sext i32 %12 to i64
  %14 = getelementptr inbounds [10 x i32], [10 x i32]* %2, i64 0, i64 %13
  store i32 %11, i32* %14, align 4
  br label %15

15:                                               ; preds = %9
  %16 = load i32, i32* %3, align 4
  %17 = add nsw i32 %16, 1
  store i32 %17, i32* %3, align 4
  br label %6

18:                                               ; preds = %6
  store i32 0, i32* %4, align 4
  store i32 0, i32* %5, align 4
  br label %19

19:                                               ; preds = %29, %18
  %20 = load i32, i32* %5, align 4
  %21 = icmp slt i32 %20, 10
  br i1 %21, label %22, label %32

22:                                               ; preds = %19
  %23 = load i32, i32* %5, align 4
  %24 = sext i32 %23 to i64
  %25 = getelementptr inbounds [10 x i32], [10 x i32]* %2, i64 0, i64 %24
  %26 = load i32, i32* %25, align 4
  %27 = load i32, i32* %4, align 4
  %28 = add nsw i32 %27, %26
  store i32 %28, i32* %4, align 4
  br label %29

29:                                               ; preds = %22
  %30 = load i32, i32* %5, align 4
  %31 = add nsw i32 %30, 1
  store i32 %31, i32* %5, align 4
  br label %19

32:                                               ; preds = %19
  %33 = load i32, i32* %4, align 4
  ret i32 %33
}

attributes #0 = { noinline nounwind optnone uwtable "correctly-rounded-divide-sqrt-fp-math"="false" "disable-tail-calls"="false" "frame-pointer"="all" "less-precise-fpmad"="false" "min-legal-vector-width"="0" "no-infs-fp-math"="false" "no-jump-tables"="false" "no-nans-fp-math"="false" "no-signed-zeros-fp-math"="false" "no-trapping-math"="false" "stack-protector-buffer-size"="8" "target-cpu"="x86-64" "target-features"="+cx8,+fxsr,+mmx,+sse,+sse2,+x87" "unsafe-fp-math"="false" "use-soft-float"="false" }
    "#;
    assert_eq!(run(asm, vec![]), GenericValue::Int32(55));
}

#[test]
fn exec8() {
    let asm = r#"
source_filename = "c.c"
target datalayout = "e-m:e-p270:32:32-p271:32:32-p272:64:64-i64:64-f80:128-n8:16:32:64-S128"
target triple = "x86_64-pc-linux-gnu"

@.str = private unnamed_addr constant [12 x i8] c"hello world\00", align 1

; Function Attrs: noinline nounwind optnone uwtable
define dso_local i32 @main() #0 {
  %1 = alloca i32, align 4
  %2 = alloca i8*, align 8
  store i32 0, i32* %1, align 4
  store i8* getelementptr inbounds ([12 x i8], [12 x i8]* @.str, i64 0, i64 0), i8** %2, align 8
  %3 = load i8*, i8** %2, align 8
  %4 = getelementptr inbounds i8, i8* %3, i64 1
  %5 = load i8, i8* %4, align 1
  %6 = sext i8 %5 to i32
  ret i32 %6
}

attributes #0 = { noinline nounwind optnone uwtable "correctly-rounded-divide-sqrt-fp-math"="false" "disable-tail-calls"="false" "frame-pointer"="all" "less-precise-fpmad"="false" "min-legal-vector-width"="0" "no-infs-fp-math"="false" "no-jump-tables"="false" "no-nans-fp-math"="false" "no-signed-zeros-fp-math"="false" "no-trapping-math"="false" "stack-protector-buffer-size"="8" "target-cpu"="x86-64" "target-features"="+cx8,+fxsr,+mmx,+sse,+sse2,+x87" "unsafe-fp-math"="false" "use-soft-float"="false" }
    "#;
    assert_eq!(run(asm, vec![]), GenericValue::Int32(101));
}

macro_rules! icmp_test {
    ($name:ident, $op:expr, $cases:expr) => {
        #[test]
        fn $name() {
            let asm = format!(
                r#"
            define dso_local i32 @main(i32 %arg) #0 {{
              %t = icmp {} i32 %arg, 0
              br i1 %t, label %l1, label %l2
            l1:
              ret i32 0
            l2:
              ret i32 1
            }}
            "#,
                $op
            );
            for (arg, expected) in $cases {
                assert_eq!(
                    run(&asm, vec![GenericValue::Int32(arg)]), // ok -> 0, err -> 1
                    GenericValue::Int32(expected)
                );
            }
        }
    };
}

// TODO: We need a better way for testing operators.
icmp_test!(exec_icmp_eq, "eq", [(0, 0), (1, 1)]);
icmp_test!(exec_icmp_ne, "ne", [(0, 1), (1, 0)]);
icmp_test!(exec_icmp_slt, "slt", [(0, 1), (-1, 0), (1, 1)]);
icmp_test!(exec_icmp_sle, "sle", [(0, 0), (-1, 0), (1, 1)]);
icmp_test!(exec_icmp_sgt, "sgt", [(0, 1), (-1, 1), (1, 0)]);
icmp_test!(exec_icmp_sge, "sge", [(0, 0), (-1, 1), (1, 0)]);
icmp_test!(exec_icmp_ult, "ult", [(0, 1), (1, 1)]);
icmp_test!(exec_icmp_ule, "ule", [(0, 0), (1, 1)]);
icmp_test!(exec_icmp_ugt, "ugt", [(0, 1), (1, 0)]);
icmp_test!(exec_icmp_uge, "uge", [(0, 0), (1, 0)]);

// #[cfg(target_os = "linux")]
// #[test]
// fn exec9() {
//     let asm = r#"
// source_filename = "c.c"
// target datalayout = "e-m:e-p270:32:32-p271:32:32-p272:64:64-i64:64-f80:128-n8:16:32:64-S128"
// target triple = "x86_64-pc-linux-gnu"
//
// @.str = private unnamed_addr constant [12 x i8] c"hello world\00", align 1
//
// ; Function Attrs: noinline nounwind optnone uwtable
// define dso_local i32 @main() #0 {
//   %1 = alloca i32, align 4
//   store i32 0, i32* %1, align 4
//   %2 = call i32 @puts(i8* getelementptr inbounds ([12 x i8], [12 x i8]* @.str, i64 0, i64 0))
//   ret i32 0
// }
//
// declare dso_local i32 @puts(i8*) #1
//
// attributes #0 = { noinline nounwind optnone uwtable "correctly-rounded-divide-sqrt-fp-math"="false" "disable-tail-calls"="false" "frame-pointer"="all" "less-precise-fpmad"="false" "min-legal-vector-width"="0" "no-infs-fp-math"="false" "no-jump-tables"="false" "no-nans-fp-math"="false" "no-signed-zeros-fp-math"="false" "no-trapping-math"="false" "stack-protector-buffer-size"="8" "target-cpu"="x86-64" "target-features"="+cx8,+fxsr,+mmx,+sse,+sse2,+x87" "unsafe-fp-math"="false" "use-soft-float"="false" }
// attributes #1 = { "correctly-rounded-divide-sqrt-fp-math"="false" "disable-tail-calls"="false" "frame-pointer"="all" "less-precise-fpmad"="false" "no-infs-fp-math"="false" "no-nans-fp-math"="false" "no-signed-zeros-fp-math"="false" "no-trapping-math"="false" "stack-protector-buffer-size"="8" "target-cpu"="x86-64" "target-features"="+cx8,+fxsr,+mmx,+sse,+sse2,+x87" "unsafe-fp-math"="false" "use-soft-float"="false" }
//
//     "#;
//     let module = module::parse_assembly(asm).unwrap();
//     let ctx = interpreter::Context::new(&module)
//         .with_lib("/lib/x86_64-linux-gnu/libc.so.6")
//         .expect("failed to load libc");
//     let main = module.find_function_by_name("main").unwrap();
//     assert_eq!(
//         interpreter::run_function(&ctx, main, vec![]).unwrap(),
//         GenericValue::Int32(0),
//     );
// }
//
// #[cfg(target_os = "linux")]
// #[test]
// fn exec10() {
//     for (x, y, z, op) in vec![
//         (2, 3, 5, "add"),
//         (3, 30, -27, "sub"),
//         (5, 23, 115, "mul"),
//         (123, 23, 5, "sdiv"),
//         (39, 30, 9, "srem"),
//     ] {
//         let asm = format!(
//             "
//     define dso_local i32 @f(i32 %0, i32 %1) {{
//       %3 = {} i32 %0, %1
//       ret i32 %3
//     }}",
//             op
//         );
//         let module = module::parse_assembly(asm.as_str()).unwrap();
//         let ctx = interpreter::Context::new(&module)
//             .with_lib("/lib/x86_64-linux-gnu/libc.so.6")
//             .expect("failed to load libc");
//         let main = module.find_function_by_name("f").unwrap();
//         assert_eq!(
//             interpreter::run_function(
//                 &ctx,
//                 main,
//                 vec![GenericValue::Int32(x), GenericValue::Int32(y)]
//             )
//             .unwrap(),
//             GenericValue::Int32(z),
//         );
//     }
// }

#[test]
fn exec_cstr() {
  let asm = r#"
  @.str = private unnamed_addr constant [5 x i8] c"test\00"
  define i8* @f() {
      ret i8* getelementptr inbounds ([5 x i8], [5 x i8]* @.str, i64 0, i64 0)
  }
  "#;
  let rc = run_libc(asm,"f",vec![]);
  let str_ = unsafe { std::ffi::CStr::from_ptr(rc.to_ptr().unwrap() as *mut i8) }.to_str().unwrap();
  assert_eq!(str_,"test");
}

#[test]
fn exec_fprintf() {
  let asm = r#"
  @.str = private unnamed_addr constant [9 x i8] c"test.txt\00", align 8
  @.str.1 = private unnamed_addr constant [2 x i8] c"w\00", align 8
  @.str.2 = private unnamed_addr constant [12 x i8] c"%d %d %d %d\00", align 8
  @.str.3 = private unnamed_addr constant [2 x i8] c"r\00", align 8
  @buf = common global [256 x i8] zeroinitializer, align 8

  define i8* @f() {
    %fpos_ptr = alloca i64, align 8
    %fp = call i8* @fopen(i8* getelementptr inbounds ([9 x i8], [9 x i8]* @.str, i64 0, i64 0), i8* getelementptr inbounds ([2 x i8], [2 x i8]* @.str.1, i64 0, i64 0))
    %rc1 = call i32 @fprintf(i8* %fp, i8* getelementptr inbounds ([12 x i8], [12 x i8]* @.str.2, i64 0, i64 0), i32 11, i32 22, i32 33, i32 44)
    %rc2 = call i32 @fgetpos(i8* %fp, i64* %fpos_ptr)
    %rc3 = call i32 @fclose(i8* %fp)
    %fp2 = call i8* @fopen(i8* getelementptr inbounds ([9 x i8], [9 x i8]* @.str, i64 0, i64 0), i8* getelementptr inbounds ([2 x i8], [2 x i8]* @.str.3, i64 0, i64 0))
    %rc4 = call i32 @fseek(i8* %fp2, i64 0, i32 0)
    %fpos = load i64, i64* %fpos_ptr, align 8
    %len = call i64 @fread(i8* getelementptr inbounds ([256 x i8], [256 x i8]* @buf, i64 0, i64 0), i64 1, i64 %fpos, i8* %fp2)
    %len2 = trunc i64 %len to i32
    %len3 = sext i32 %len2 to i64
    %str_end = getelementptr inbounds i8, i8* getelementptr inbounds ([256 x i8], [256 x i8]* @buf, i64 0, i64 0), i64 %len3
    store i8 0, i8* %str_end, align 1
    %rc5 = call i32 @fclose(i8* %fp2)
    %rc6 = call i32 @unlink(i8* getelementptr inbounds ([9 x i8], [9 x i8]* @.str, i64 0, i64 0))
    ret i8* getelementptr inbounds ([256 x i8], [256 x i8]* @buf, i64 0, i64 0)
      ret i8* @buf
  }


  declare i8* @"fopen"(i8*, i8*)
  declare i32 @fprintf(i8*, i8*, ...)
  declare i32 @fgetpos(i8*, i64*)
  declare i32 @fclose(i8*)
  declare i32 @fseek(i8*, i64, i32)
  declare i64 @fread(i8*, i64, i64, i8*)
  declare i32 @unlink(i8*)
  "#;
  let rc = run_libc(asm,"f",vec![]);
  let str_ = unsafe { std::ffi::CStr::from_ptr(rc.to_ptr().unwrap() as *mut i8) }.to_str().unwrap();
  assert_eq!(str_,"11 22 33 44");
}

#[test]
fn exec_sscanf() {
    let asm = 
      r#"
      @.str = private unnamed_addr constant [3 x i8] c"11\00", align 1
      @.str.1 = private unnamed_addr constant [3 x i8] c"%d\00", align 1
      define i32 @f() #0 {
        %a = alloca i32, align 8
        %b = call i32 (i8*, i8*, ...) @sscanf(i8* getelementptr inbounds ([3 x i8], [3 x i8]* @.str, i64 0, i64 0),
                                              i8* getelementptr inbounds ([3 x i8], [3 x i8]* @.str.1, i64 0, i64 0), i32* %a)
        %c = load i32, i32* %a, align 8
        ret i32 %c
      }
      declare i32 @sscanf(i8*, i8*, ...)
      "#;
      assert_eq!(run_libc(asm,"f",vec![]), GenericValue::Int32(11));
}

#[test]
fn exec_sprintf() {
    let asm = 
      r#"
      @.str = private unnamed_addr constant [12 x i8] c"%d %d %d %d\00", align 1
      @buf = common global [26 x i8] zeroinitializer, align 1
      define i8* @f() {
        %rc2 = call i32 @sprintf(i8* getelementptr inbounds ([26 x i8], [26 x i8]* @buf, i64 0, i64 0),
                                 i8* getelementptr inbounds ([12 x i8], [12 x i8]* @.str, i64 0, i64 0),
                                 i32 12,i32 34,i32 56,i32 78)
        ret i8* getelementptr inbounds ([26 x i8], [26 x i8]* @buf, i64 0, i64 0)
      }
      declare i32 @sprintf(i8*, i8*, ...)
      "#;
      let rc = run_libc(asm,"f",vec![]);
      let str_ = unsafe { std::ffi::CStr::from_ptr(rc.to_ptr().unwrap() as *mut i8) }.to_str().unwrap();
      assert_eq!(str_,"12 34 56 78");
}

#[test]
fn exec_array_load_store() {
    let asm = 
      r#"
      @buf = common global [26 x i8] zeroinitializer, align 1
      define i8* @f() {
        store i8 118, i8* getelementptr inbounds ([26 x i8], [26 x i8]* @buf, i64 0, i64 0)
        store i8 119, i8* getelementptr inbounds ([26 x i8], [26 x i8]* @buf, i64 0, i64 1)
        %w = load i8, i8* getelementptr inbounds ([26 x i8], [26 x i8]* @buf, i64 0, i64 1)
        store i8 %w, i8* getelementptr inbounds ([26 x i8], [26 x i8]* @buf, i64 0, i64 2)
        ret i8* getelementptr inbounds ([26 x i8], [26 x i8]* @buf, i64 0, i64 0)
      }
      "#;
      let rc = run_libc(asm,"f",vec![]);
      let str_ = unsafe { std::ffi::CStr::from_ptr(rc.to_ptr().unwrap() as *mut i8) }.to_str().unwrap();
      assert_eq!(str_,"vww");
}

#[cfg(test)]
fn run(asm: &str, args: Vec<GenericValue>) -> GenericValue {
    let module = module::parse_assembly(asm).unwrap();
    let ctx = interpreter::Context::new(&module);
    let main = module.find_function_by_name("main").unwrap();
    interpreter::run_function(&ctx, main, args).unwrap()
}

#[cfg(test)]
fn run_libc(asm: &str, fname: &str,args: Vec<GenericValue>) -> GenericValue {
    let module = module::parse_assembly(asm).unwrap();
    let mut ctx = interpreter::Context::new(&module);
    #[cfg(target_os = "macos")]
    {ctx = ctx.with_lib("libc.dylib").expect("failed to load libc");}
    #[cfg(target_os = "linux")]
    {ctx = ctx.with_lib("libc.so.6").expect("failed to load libc");}
    #[cfg(target_os = "windows")]
    {ctx = ctx.with_lib("msvcrt.dll").expect("failed to load msvcrt.dll");}
    let main = module.find_function_by_name(fname).unwrap();
    interpreter::run_function(&ctx, main, args).unwrap()
}
