use std::os::raw::c_char;

use vicis_core::ir::module::Module;
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

#[test]
fn exec_cstr() {
    let asm = r#"
  @.str = private unnamed_addr constant [5 x i8] c"test\00"
  define i8* @f() {
      ret i8* getelementptr inbounds ([5 x i8], [5 x i8]* @.str, i64 0, i64 0)
  }
  "#;
    let rc = run_libc(asm, "f", vec![]);
    let str_ = unsafe { std::ffi::CStr::from_ptr(rc.to_ptr().unwrap() as *mut c_char) }
        .to_str()
        .unwrap();
    assert_eq!(str_, "test");
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
    let rc = run_libc(asm, "f", vec![]);
    let str_ = unsafe { std::ffi::CStr::from_ptr(rc.to_ptr().unwrap() as *mut c_char) }
        .to_str()
        .unwrap();
    assert_eq!(str_, "11 22 33 44");
}

#[test]
fn exec_sscanf() {
    let asm = r#"
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
    assert_eq!(run_libc(asm, "f", vec![]), GenericValue::Int32(11));
}

#[test]
fn exec_sprintf() {
    let asm = r#"
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
    let rc = run_libc(asm, "f", vec![]);
    let str_ = unsafe { std::ffi::CStr::from_ptr(rc.to_ptr().unwrap() as *mut c_char) }
        .to_str()
        .unwrap();
    assert_eq!(str_, "12 34 56 78");
}

#[test]
fn exec_gep() {
    let asm = r#"
%struct.type = type { i8, %struct.type2, i32 }
%struct.type2 = type { i32, i64, [3 x i8] }

@.str = private unnamed_addr constant [18 x i8] c"%d %d %lld %s %d\0A\00", align 1
@buf = common global [128 x i8] zeroinitializer, align 1

define dso_local i8* @f() #0 {
  %1 = alloca i32, align 4
  %2 = alloca %struct.type, align 8
  store i32 0, i32* %1, align 4
  %3 = getelementptr inbounds %struct.type, %struct.type* %2, i32 0, i32 0
  store i8 65, i8* %3, align 8
  %4 = getelementptr inbounds %struct.type, %struct.type* %2, i32 0, i32 1
  %5 = getelementptr inbounds %struct.type2, %struct.type2* %4, i32 0, i32 0
  store i32 123, i32* %5, align 8
  %6 = getelementptr inbounds %struct.type, %struct.type* %2, i32 0, i32 1
  %7 = getelementptr inbounds %struct.type2, %struct.type2* %6, i32 0, i32 1
  store i64 12345678900, i64* %7, align 8
  %8 = getelementptr inbounds %struct.type, %struct.type* %2, i32 0, i32 1
  %9 = getelementptr inbounds %struct.type2, %struct.type2* %8, i32 0, i32 2
  %10 = getelementptr inbounds [3 x i8], [3 x i8]* %9, i64 0, i64 0
  store i8 104, i8* %10, align 8
  %11 = getelementptr inbounds %struct.type, %struct.type* %2, i32 0, i32 1
  %12 = getelementptr inbounds %struct.type2, %struct.type2* %11, i32 0, i32 2
  %13 = getelementptr inbounds [3 x i8], [3 x i8]* %12, i64 0, i64 1
  store i8 105, i8* %13, align 1
  %14 = getelementptr inbounds %struct.type, %struct.type* %2, i32 0, i32 1
  %15 = getelementptr inbounds %struct.type2, %struct.type2* %14, i32 0, i32 2
  %16 = getelementptr inbounds [3 x i8], [3 x i8]* %15, i64 0, i64 2
  store i8 0, i8* %16, align 2
  %17 = getelementptr inbounds %struct.type, %struct.type* %2, i32 0, i32 2
  store i32 456, i32* %17, align 8
  %18 = getelementptr inbounds %struct.type, %struct.type* %2, i32 0, i32 0
  %19 = load i8, i8* %18, align 8
  %20 = sext i8 %19 to i32
  %21 = getelementptr inbounds %struct.type, %struct.type* %2, i32 0, i32 1
  %22 = getelementptr inbounds %struct.type2, %struct.type2* %21, i32 0, i32 0
  %23 = load i32, i32* %22, align 8
  %24 = getelementptr inbounds %struct.type, %struct.type* %2, i32 0, i32 1
  %25 = getelementptr inbounds %struct.type2, %struct.type2* %24, i32 0, i32 1
  %26 = load i64, i64* %25, align 8
  %27 = getelementptr inbounds %struct.type, %struct.type* %2, i32 0, i32 1
  %28 = getelementptr inbounds %struct.type2, %struct.type2* %27, i32 0, i32 2
  %29 = getelementptr inbounds [3 x i8], [3 x i8]* %28, i64 0, i64 0
  %30 = getelementptr inbounds %struct.type, %struct.type* %2, i32 0, i32 2
  %31 = load i32, i32* %30, align 8
  %32 = call i32 (i8*, i8*, ...) @sprintf(
      i8* getelementptr inbounds ([128 x i8], [128 x i8]* @buf, i64 0, i64 0),
      i8* getelementptr inbounds ([18 x i8], [18 x i8]* @.str, i64 0, i64 0),
      i32 %20, i32 %23, i64 %26, i8* %29, i32 %31)
  ret i8* getelementptr inbounds ([128 x i8], [128 x i8]* @buf, i64 0, i64 0)
}

declare i32 @sprintf(i8*, i8*, ...)
      "#;
    let v = run_libc(asm, "f", vec![]);
    let str_ = unsafe { std::ffi::CStr::from_ptr(v.to_ptr().unwrap() as *mut c_char) }
        .to_str()
        .unwrap();
    assert_eq!(str_, "65 123 12345678900 hi 456\n");
}

#[test]
fn exec_array_load_store() {
    let asm = r#"
      @buf = common global [26 x i8] zeroinitializer, align 1
      define i8* @f() {
        store i8 118, i8* getelementptr inbounds ([26 x i8], [26 x i8]* @buf, i64 0, i64 0)
        store i8 119, i8* getelementptr inbounds ([26 x i8], [26 x i8]* @buf, i64 0, i64 1)
        %w = load i8, i8* getelementptr inbounds ([26 x i8], [26 x i8]* @buf, i64 0, i64 1)
        store i8 %w, i8* getelementptr inbounds ([26 x i8], [26 x i8]* @buf, i64 0, i64 2)
        ret i8* getelementptr inbounds ([26 x i8], [26 x i8]* @buf, i64 0, i64 0)
      }
      "#;
    let rc = run_libc(asm, "f", vec![]);
    let str_ = unsafe { std::ffi::CStr::from_ptr(rc.to_ptr().unwrap() as *mut c_char) }
        .to_str()
        .unwrap();
    assert_eq!(str_, "vww");
}

#[test]
fn exec_test_phi() {
    let asm = r#"
    define i32 @main() {
    enter1:
      %result_0 = add i32 0, 1
      %i_0 = add i32 0, 8
      br label %for.cond.2
    for.cond.2:
      %i_1 = phi i32 [%i_0, %enter1], [%v12_1, %for.body.2]
      %result_1 = phi i32 [%result_0, %enter1], [%v9_1, %for.body.2]
      %v5_1 = add i32 0, 0
      %v6_1 = icmp sgt i32 %i_1, %v5_1
      %0 = icmp ne i1 %v6_1, 0
      br i1 %0, label %for.body.2, label %for.end.2
    for.body.2:
      %v9_1 = mul i32 %result_1, %i_1
      %v11_1 = add i32 0, 1
      %v12_1 = sub i32 %i_1, %v11_1
      br label %for.cond.2
    for.end.2:
      ret i32 %result_1
    }
    "#;
    let rc = run(asm, vec![]);
    assert_eq!(rc, GenericValue::Int32(40320));
}

#[test]
fn exec_test_memcpy() {
    let asm = r#"
@__const.main.a = private unnamed_addr constant [5 x i32] [i32 1, i32 2, i32 3, i32 4, i32 5], align 16

define dso_local i32 @main() {
  %1 = alloca i32, align 4
  %2 = alloca [5 x i32], align 16
  store i32 0, i32* %1, align 4
  %3 = bitcast [5 x i32]* %2 to i8*
  call void @llvm.memcpy.p0i8.p0i8.i64(i8* align 16 %3, i8* align 16 bitcast ([5 x i32]* @__const.main.a to i8*), i64 20, i1 false)
  %4 = getelementptr inbounds [5 x i32], [5 x i32]* %2, i64 0, i64 0
  %5 = load i32, i32* %4, align 16
  %6 = getelementptr inbounds [5 x i32], [5 x i32]* %2, i64 0, i64 1
  %7 = load i32, i32* %6, align 4
  %8 = add nsw i32 %5, %7
  %9 = getelementptr inbounds [5 x i32], [5 x i32]* %2, i64 0, i64 2
  %10 = load i32, i32* %9, align 8
  %11 = add nsw i32 %8, %10
  %12 = getelementptr inbounds [5 x i32], [5 x i32]* %2, i64 0, i64 3
  %13 = load i32, i32* %12, align 4
  %14 = add nsw i32 %11, %13
  %15 = getelementptr inbounds [5 x i32], [5 x i32]* %2, i64 0, i64 4
  %16 = load i32, i32* %15, align 16
  %17 = add nsw i32 %14, %16
  ret i32 %17
}

declare void @llvm.memcpy.p0i8.p0i8.i64(i8* noalias nocapture writeonly, i8* noalias nocapture readonly, i64, i1 immarg)
"#;
    let rc = run(asm, vec![]);
    assert_eq!(rc, GenericValue::Int32(15));
}

#[cfg(test)]
fn run(asm: &str, args: Vec<GenericValue>) -> GenericValue {
    let module = Module::try_from(asm).unwrap();
    let ctx = interpreter::ContextBuilder::new(&module).build().unwrap();
    let main = module.find_function_by_name("main").unwrap();
    interpreter::run_function(&ctx, main, args).unwrap()
}

#[cfg(test)]
fn run_libc(asm: &str, fname: &str, args: Vec<GenericValue>) -> GenericValue {
    let module = Module::try_from(asm).unwrap();
    let mut ctx_builder = interpreter::ContextBuilder::new(&module);
    #[cfg(target_os = "macos")]
    {
        ctx_builder = ctx_builder.with_lib("libc.dylib");
    }
    #[cfg(target_os = "linux")]
    {
        ctx_builder = ctx_builder.with_lib("libc.so.6");
    }
    #[cfg(target_os = "windows")]
    {
        ctx_builder = ctx_builder.with_lib("msvcrt.dll");
    }
    let ctx = ctx_builder.build().unwrap();
    let main = module.find_function_by_name(fname).unwrap();
    interpreter::run_function(&ctx, main, args).unwrap()
}
