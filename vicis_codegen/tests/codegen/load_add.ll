source_filename = "asm.c"                                                                          
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
