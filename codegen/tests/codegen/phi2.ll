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
