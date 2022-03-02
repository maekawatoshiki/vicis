; ModuleID = 'x.c'
source_filename = "x.c"
target datalayout = "e-m:e-p270:32:32-p271:32:32-p272:64:64-i64:64-f80:128-n8:16:32:64-S128"
target triple = "x86_64-pc-linux-gnu"

%struct.type = type { i8, %struct.type2, i32 }
%struct.type2 = type { i32, i64, [3 x i8] }

@.str = private unnamed_addr constant [18 x i8] c"%d %d %lld %s %d\0A\00", align 1

; Function Attrs: noinline nounwind optnone uwtable
define dso_local i32 @main() #0 {
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
  %32 = call i32 (i8*, ...) @printf(i8* getelementptr inbounds ([18 x i8], [18 x i8]* @.str, i64 0, i64 0), i32 %20, i32 %23, i64 %26, i8* %29, i32 %31)
  ret i32 0
}

declare dso_local i32 @printf(i8*, ...) #1

attributes #0 = { noinline nounwind optnone uwtable "correctly-rounded-divide-sqrt-fp-math"="false" "disable-tail-calls"="false" "frame-pointer"="all" "less-precise-fpmad"="false" "min-legal-vector-width"="0" "no-infs-fp-math"="false" "no-jump-tables"="false" "no-nans-fp-math"="false" "no-signed-zeros-fp-math"="false" "no-trapping-math"="false" "stack-protector-buffer-size"="8" "target-cpu"="x86-64" "target-features"="+cx8,+fxsr,+mmx,+sse,+sse2,+x87" "unsafe-fp-math"="false" "use-soft-float"="false" }
attributes #1 = { "correctly-rounded-divide-sqrt-fp-math"="false" "disable-tail-calls"="false" "frame-pointer"="all" "less-precise-fpmad"="false" "no-infs-fp-math"="false" "no-nans-fp-math"="false" "no-signed-zeros-fp-math"="false" "no-trapping-math"="false" "stack-protector-buffer-size"="8" "target-cpu"="x86-64" "target-features"="+cx8,+fxsr,+mmx,+sse,+sse2,+x87" "unsafe-fp-math"="false" "use-soft-float"="false" }

!llvm.module.flags = !{!0}
!llvm.ident = !{!1}

!0 = !{i32 1, !"wchar_size", i32 4}
!1 = !{!"clang version 10.0.0-4ubuntu1 "}
