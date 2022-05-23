source_filename = "spill"
target datalayout = "e-m:e-p270:32:32-p271:32:32-p272:64:64-i64:64-f80:128-n8:16:32:64-S128"
target triple = "x86_64-pc-linux-gnu"

define i32 @main() {
  %a1 = alloca i32
  %a2 = alloca i32
  %a3 = alloca i32
  %a4 = alloca i32
  %a5 = alloca i32
  %a6 = alloca i32
  %a7 = alloca i32
  %a8 = alloca i32
  %a9 = alloca i32
  %a10 = alloca i32
  store i32 1, i32* %a1
  store i32 2, i32* %a2
  store i32 3, i32* %a3
  store i32 4, i32* %a4
  store i32 5, i32* %a5
  store i32 6, i32* %a6
  store i32 7, i32* %a7
  store i32 8, i32* %a8
  store i32 9, i32* %a9
  store i32 10, i32* %a10
  %a11 = load i32, i32* %a1
  %a12 = load i32, i32* %a2
  %a13 = load i32, i32* %a3
  %a14 = load i32, i32* %a4
  %a15 = load i32, i32* %a5
  %a16 = load i32, i32* %a6
  %a17 = load i32, i32* %a7
  %a18 = load i32, i32* %a8
  %a19 = load i32, i32* %a9
  %a20 = load i32, i32* %a10
  %a21 = add nsw i32 %a11, %a12
  %a22 = add nsw i32 %a21, %a13
  %a23 = add nsw i32 %a22, %a14
  %a24 = add nsw i32 %a23, %a15
  %a25 = add nsw i32 %a24, %a16
  %a26 = add nsw i32 %a25, %a17
  %a27 = add nsw i32 %a26, %a18
  %a28 = add nsw i32 %a27, %a19
  %a29 = add nsw i32 %a28, %a20
  ret i32 %a29
}
