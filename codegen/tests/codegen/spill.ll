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
  %a11 = alloca i32
  %a12 = alloca i32
  %a13 = alloca i32
  %a14 = alloca i32
  %a15 = alloca i32
  %a16 = alloca i32
  %a17 = alloca i32
  %a18 = alloca i32
  %a19 = alloca i32
  %a20 = alloca i32
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
  store i32 11, i32* %a11
  store i32 12, i32* %a12
  store i32 13, i32* %a13
  store i32 14, i32* %a14
  store i32 15, i32* %a15
  store i32 16, i32* %a16
  store i32 17, i32* %a17
  store i32 18, i32* %a18
  store i32 19, i32* %a19
  store i32 20, i32* %a20
  %a21 = load i32, i32* %a1
  %a22 = load i32, i32* %a2
  %a23 = load i32, i32* %a3
  %a24 = load i32, i32* %a4
  %a25 = load i32, i32* %a5
  %a26 = load i32, i32* %a6
  %a27 = load i32, i32* %a7
  %a28 = load i32, i32* %a8
  %a29 = load i32, i32* %a9
  %a30 = load i32, i32* %a10
  %a31 = load i32, i32* %a11
  %a32 = load i32, i32* %a12
  %a33 = load i32, i32* %a13
  %a34 = load i32, i32* %a14
  %a35 = load i32, i32* %a15
  %a36 = load i32, i32* %a16
  %a37 = load i32, i32* %a17
  %a38 = load i32, i32* %a18
  %a39 = load i32, i32* %a19
  %a40 = load i32, i32* %a20
  %a41 = add nsw i32 %a21, %a22
  %a42 = add nsw i32 %a41, %a23
  %a43 = add nsw i32 %a42, %a24
  %a44 = add nsw i32 %a43, %a25
  %a45 = add nsw i32 %a44, %a26
  %a46 = add nsw i32 %a45, %a27
  %a47 = add nsw i32 %a46, %a28
  %a48 = add nsw i32 %a47, %a29
  %a49 = add nsw i32 %a48, %a30
  %a50 = add nsw i32 %a49, %a31
  %a51 = add nsw i32 %a50, %a32
  %a52 = add nsw i32 %a51, %a33
  %a53 = add nsw i32 %a52, %a34
  %a54 = add nsw i32 %a53, %a35
  %a55 = add nsw i32 %a54, %a36
  %a56 = add nsw i32 %a55, %a37
  %a57 = add nsw i32 %a56, %a38
  %a58 = add nsw i32 %a57, %a39
  %a59 = add nsw i32 %a58, %a40
  ret i32 %a59
}
