define dso_local i32 @f(i32 %a) #0 {
  ret i32 %a
}

; Function Attrs: noinline nounwind optnone uwtable
define dso_local i32 @main() #0 {
  %1 = call i32 @f(i32 1)
  ret i32 %1
}
