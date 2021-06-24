; ModuleID = 'a.7rcbfp3g-cgu.0'
source_filename = "a.7rcbfp3g-cgu.0"
target datalayout = "e-m:e-p270:32:32-p271:32:32-p272:64:64-i64:64-f80:128-n8:16:32:64-S128"
target triple = "x86_64-pc-linux-gnu"

%d = type { i8, %a }
%a = type { i32, [123 x i64] }
%b = type { i32, { i32 } }
%c = type { %c*, i8 }
%e = type i64
%"あいうえお" = type { i32 }

define dso_local i32 @main() {
  %1 = alloca %a
  %2 = alloca %b
  %3 = alloca %c
  %4 = alloca %d
  %5 = alloca %e
  %6 = alloca %"あいうえお"
  ret i32 0
}

