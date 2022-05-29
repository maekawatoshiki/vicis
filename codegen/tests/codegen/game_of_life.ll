; ModuleID = 'game_of_life.ll'
source_filename = "game_of_life.c"
target datalayout = "e-m:e-p270:32:32-p271:32:32-p272:64:64-i64:64-f80:128-n8:16:32:64-S128"
target triple = "x86_64-pc-linux-gnu"

@__const.main.grid = private unnamed_addr constant <{ [20 x i32], [20 x i32], [20 x i32], [20 x i32], [20 x i32], [20 x i32], [20 x i32], [20 x i32], [20 x i32], [20 x i32], [20 x i32], [20 x i32], [20 x i32], [20 x i32], <{ i32, i32, i32, i32, i32, i32, [14 x i32] }>, <{ i32, i32, i32, i32, i32, i32, [14 x i32] }>, <{ i32, i32, i32, i32, i32, i32, [14 x i32] }>, <{ i32, i32, i32, i32, i32, [15 x i32] }>, [20 x i32], [20 x i32] }> <{ [20 x i32] zeroinitializer, [20 x i32] zeroinitializer, [20 x i32] zeroinitializer, [20 x i32] zeroinitializer, [20 x i32] zeroinitializer, [20 x i32] zeroinitializer, [20 x i32] zeroinitializer, [20 x i32] [i32 0, i32 0, i32 0, i32 0, i32 0, i32 1, i32 1, i32 1, i32 1, i32 1, i32 1, i32 1, i32 1, i32 1, i32 1, i32 0, i32 0, i32 0, i32 0, i32 0], [20 x i32] zeroinitializer, [20 x i32] zeroinitializer, [20 x i32] zeroinitializer, [20 x i32] zeroinitializer, [20 x i32] zeroinitializer, [20 x i32] zeroinitializer, <{ i32, i32, i32, i32, i32, i32, [14 x i32] }> <{ i32 0, i32 0, i32 1, i32 1, i32 1, i32 1, [14 x i32] zeroinitializer }>, <{ i32, i32, i32, i32, i32, i32, [14 x i32] }> <{ i32 0, i32 1, i32 0, i32 0, i32 0, i32 1, [14 x i32] zeroinitializer }>, <{ i32, i32, i32, i32, i32, i32, [14 x i32] }> <{ i32 0, i32 0, i32 0, i32 0, i32 0, i32 1, [14 x i32] zeroinitializer }>, <{ i32, i32, i32, i32, i32, [15 x i32] }> <{ i32 0, i32 1, i32 0, i32 0, i32 1, [15 x i32] zeroinitializer }>, [20 x i32] zeroinitializer, [20 x i32] zeroinitializer }>, align 16
@.str = private unnamed_addr constant [7 x i8] c"\1B[0;0H\00", align 1
@.str.1 = private unnamed_addr constant [2 x i8] c"\0A\00", align 1
@.str.2 = private unnamed_addr constant [11 x i8] c"\1B[42m  \1B[m\00", align 1
@.str.3 = private unnamed_addr constant [11 x i8] c"\1B[47m  \1B[m\00", align 1

; Function Attrs: noinline nounwind optnone uwtable
define dso_local i32 @main() #0 {
  %1 = alloca i32, align 4
  %2 = alloca [20 x [20 x i32]], align 16
  %3 = alloca [20 x [20 x i32]], align 16
  %4 = alloca i32, align 4
  %5 = alloca i32, align 4
  %6 = alloca i32, align 4
  store i32 0, i32* %1, align 4
  %7 = bitcast [20 x [20 x i32]]* %3 to i8*
  call void @llvm.memcpy.p0i8.p0i8.i64(i8* align 16 %7, i8* align 16 bitcast (<{ [20 x i32], [20 x i32], [20 x i32], [20 x i32], [20 x i32], [20 x i32], [20 x i32], [20 x i32], [20 x i32], [20 x i32], [20 x i32], [20 x i32], [20 x i32], [20 x i32], <{ i32, i32, i32, i32, i32, i32, [14 x i32] }>, <{ i32, i32, i32, i32, i32, i32, [14 x i32] }>, <{ i32, i32, i32, i32, i32, i32, [14 x i32] }>, <{ i32, i32, i32, i32, i32, [15 x i32] }>, [20 x i32], [20 x i32] }>* @__const.main.grid to i8*), i64 1600, i1 false)
  store i32 0, i32* %6, align 4
  br label %8

8:                                                ; preds = %122, %0
  %9 = load i32, i32* %6, align 4
  %10 = icmp slt i32 %9, 50
  br i1 %10, label %11, label %125

11:                                               ; preds = %8
  %12 = call i32 (i8*, ...) @printf(i8* noundef getelementptr inbounds ([7 x i8], [7 x i8]* @.str, i64 0, i64 0))
  store i32 0, i32* %4, align 4
  br label %13

13:                                               ; preds = %49, %11
  %14 = load i32, i32* %4, align 4
  %15 = icmp slt i32 %14, 20
  br i1 %15, label %16, label %52

16:                                               ; preds = %13
  %17 = call i32 (i8*, ...) @printf(i8* noundef getelementptr inbounds ([2 x i8], [2 x i8]* @.str.1, i64 0, i64 0))
  store i32 0, i32* %5, align 4
  br label %18

18:                                               ; preds = %45, %16
  %19 = load i32, i32* %5, align 4
  %20 = icmp slt i32 %19, 20
  br i1 %20, label %21, label %48

21:                                               ; preds = %18
  %22 = load i32, i32* %4, align 4
  %23 = sext i32 %22 to i64
  %24 = getelementptr inbounds [20 x [20 x i32]], [20 x [20 x i32]]* %3, i64 0, i64 %23
  %25 = load i32, i32* %5, align 4
  %26 = sext i32 %25 to i64
  %27 = getelementptr inbounds [20 x i32], [20 x i32]* %24, i64 0, i64 %26
  %28 = load i32, i32* %27, align 4
  %29 = icmp eq i32 %28, 1
  br i1 %29, label %30, label %32

30:                                               ; preds = %21
  %31 = call i32 (i8*, ...) @printf(i8* noundef getelementptr inbounds ([11 x i8], [11 x i8]* @.str.2, i64 0, i64 0))
  br label %34

32:                                               ; preds = %21
  %33 = call i32 (i8*, ...) @printf(i8* noundef getelementptr inbounds ([11 x i8], [11 x i8]* @.str.3, i64 0, i64 0))
  br label %34

34:                                               ; preds = %32, %30
  %35 = getelementptr inbounds [20 x [20 x i32]], [20 x [20 x i32]]* %3, i64 0, i64 0
  %36 = load i32, i32* %4, align 4
  %37 = load i32, i32* %5, align 4
  %38 = call i32 @count_nbr([20 x i32]* noundef %35, i32 noundef %36, i32 noundef %37, i32 noundef 20)
  %39 = load i32, i32* %4, align 4
  %40 = sext i32 %39 to i64
  %41 = getelementptr inbounds [20 x [20 x i32]], [20 x [20 x i32]]* %2, i64 0, i64 %40
  %42 = load i32, i32* %5, align 4
  %43 = sext i32 %42 to i64
  %44 = getelementptr inbounds [20 x i32], [20 x i32]* %41, i64 0, i64 %43
  store i32 %38, i32* %44, align 4
  br label %45

45:                                               ; preds = %34
  %46 = load i32, i32* %5, align 4
  %47 = add nsw i32 %46, 1
  store i32 %47, i32* %5, align 4
  br label %18, !llvm.loop !6

48:                                               ; preds = %18
  br label %49

49:                                               ; preds = %48
  %50 = load i32, i32* %4, align 4
  %51 = add nsw i32 %50, 1
  store i32 %51, i32* %4, align 4
  br label %13, !llvm.loop !8

52:                                               ; preds = %13
  store i32 0, i32* %4, align 4
  br label %53

53:                                               ; preds = %117, %52
  %54 = load i32, i32* %4, align 4
  %55 = icmp slt i32 %54, 20
  br i1 %55, label %56, label %120

56:                                               ; preds = %53
  store i32 0, i32* %5, align 4
  br label %57

57:                                               ; preds = %113, %56
  %58 = load i32, i32* %5, align 4
  %59 = icmp slt i32 %58, 20
  br i1 %59, label %60, label %116

60:                                               ; preds = %57
  %61 = load i32, i32* %4, align 4
  %62 = sext i32 %61 to i64
  %63 = getelementptr inbounds [20 x [20 x i32]], [20 x [20 x i32]]* %3, i64 0, i64 %62
  %64 = load i32, i32* %5, align 4
  %65 = sext i32 %64 to i64
  %66 = getelementptr inbounds [20 x i32], [20 x i32]* %63, i64 0, i64 %65
  %67 = load i32, i32* %66, align 4
  %68 = icmp sge i32 %67, 1
  br i1 %68, label %69, label %95

69:                                               ; preds = %60
  %70 = load i32, i32* %4, align 4
  %71 = sext i32 %70 to i64
  %72 = getelementptr inbounds [20 x [20 x i32]], [20 x [20 x i32]]* %2, i64 0, i64 %71
  %73 = load i32, i32* %5, align 4
  %74 = sext i32 %73 to i64
  %75 = getelementptr inbounds [20 x i32], [20 x i32]* %72, i64 0, i64 %74
  %76 = load i32, i32* %75, align 4
  %77 = icmp sle i32 %76, 1
  br i1 %77, label %87, label %78

78:                                               ; preds = %69
  %79 = load i32, i32* %4, align 4
  %80 = sext i32 %79 to i64
  %81 = getelementptr inbounds [20 x [20 x i32]], [20 x [20 x i32]]* %2, i64 0, i64 %80
  %82 = load i32, i32* %5, align 4
  %83 = sext i32 %82 to i64
  %84 = getelementptr inbounds [20 x i32], [20 x i32]* %81, i64 0, i64 %83
  %85 = load i32, i32* %84, align 4
  %86 = icmp sge i32 %85, 4
  br i1 %86, label %87, label %94

87:                                               ; preds = %78, %69
  %88 = load i32, i32* %4, align 4
  %89 = sext i32 %88 to i64
  %90 = getelementptr inbounds [20 x [20 x i32]], [20 x [20 x i32]]* %3, i64 0, i64 %89
  %91 = load i32, i32* %5, align 4
  %92 = sext i32 %91 to i64
  %93 = getelementptr inbounds [20 x i32], [20 x i32]* %90, i64 0, i64 %92
  store i32 0, i32* %93, align 4
  br label %94

94:                                               ; preds = %87, %78
  br label %112

95:                                               ; preds = %60
  %96 = load i32, i32* %4, align 4
  %97 = sext i32 %96 to i64
  %98 = getelementptr inbounds [20 x [20 x i32]], [20 x [20 x i32]]* %2, i64 0, i64 %97
  %99 = load i32, i32* %5, align 4
  %100 = sext i32 %99 to i64
  %101 = getelementptr inbounds [20 x i32], [20 x i32]* %98, i64 0, i64 %100
  %102 = load i32, i32* %101, align 4
  %103 = icmp eq i32 %102, 3
  br i1 %103, label %104, label %111

104:                                              ; preds = %95
  %105 = load i32, i32* %4, align 4
  %106 = sext i32 %105 to i64
  %107 = getelementptr inbounds [20 x [20 x i32]], [20 x [20 x i32]]* %3, i64 0, i64 %106
  %108 = load i32, i32* %5, align 4
  %109 = sext i32 %108 to i64
  %110 = getelementptr inbounds [20 x i32], [20 x i32]* %107, i64 0, i64 %109
  store i32 1, i32* %110, align 4
  br label %111

111:                                              ; preds = %104, %95
  br label %112

112:                                              ; preds = %111, %94
  br label %113

113:                                              ; preds = %112
  %114 = load i32, i32* %5, align 4
  %115 = add nsw i32 %114, 1
  store i32 %115, i32* %5, align 4
  br label %57, !llvm.loop !9

116:                                              ; preds = %57
  br label %117

117:                                              ; preds = %116
  %118 = load i32, i32* %4, align 4
  %119 = add nsw i32 %118, 1
  store i32 %119, i32* %4, align 4
  br label %53, !llvm.loop !10

120:                                              ; preds = %53
  %121 = call i32 @usleep(i32 noundef 100000)
  br label %122

122:                                              ; preds = %120
  %123 = load i32, i32* %6, align 4
  %124 = add nsw i32 %123, 1
  store i32 %124, i32* %6, align 4
  br label %8, !llvm.loop !11

125:                                              ; preds = %8
  ret i32 0
}

; Function Attrs: argmemonly nofree nounwind willreturn
declare void @llvm.memcpy.p0i8.p0i8.i64(i8* noalias nocapture writeonly, i8* noalias nocapture readonly, i64, i1 immarg) #1

declare i32 @printf(i8* noundef, ...) #2

; Function Attrs: noinline nounwind optnone uwtable
define dso_local i32 @count_nbr([20 x i32]* noundef %0, i32 noundef %1, i32 noundef %2, i32 noundef %3) #0 {
  %5 = alloca [20 x i32]*, align 8
  %6 = alloca i32, align 4
  %7 = alloca i32, align 4
  %8 = alloca i32, align 4
  %9 = alloca i32, align 4
  store [20 x i32]* %0, [20 x i32]** %5, align 8
  store i32 %1, i32* %6, align 4
  store i32 %2, i32* %7, align 4
  store i32 %3, i32* %8, align 4
  store i32 0, i32* %9, align 4
  %10 = load i32, i32* %6, align 4
  %11 = sub nsw i32 %10, 1
  %12 = icmp sge i32 %11, 0
  br i1 %12, label %13, label %33

13:                                               ; preds = %4
  %14 = load i32, i32* %7, align 4
  %15 = sub nsw i32 %14, 1
  %16 = icmp sge i32 %15, 0
  br i1 %16, label %17, label %33

17:                                               ; preds = %13
  %18 = load [20 x i32]*, [20 x i32]** %5, align 8
  %19 = load i32, i32* %6, align 4
  %20 = sub nsw i32 %19, 1
  %21 = sext i32 %20 to i64
  %22 = getelementptr inbounds [20 x i32], [20 x i32]* %18, i64 %21
  %23 = load i32, i32* %7, align 4
  %24 = sub nsw i32 %23, 1
  %25 = sext i32 %24 to i64
  %26 = getelementptr inbounds [20 x i32], [20 x i32]* %22, i64 0, i64 %25
  %27 = load i32, i32* %26, align 4
  %28 = icmp sge i32 %27, 1
  br i1 %28, label %29, label %32

29:                                               ; preds = %17
  %30 = load i32, i32* %9, align 4
  %31 = add nsw i32 %30, 1
  store i32 %31, i32* %9, align 4
  br label %32

32:                                               ; preds = %29, %17
  br label %33

33:                                               ; preds = %32, %13, %4
  %34 = load i32, i32* %6, align 4
  %35 = sub nsw i32 %34, 1
  %36 = icmp sge i32 %35, 0
  br i1 %36, label %37, label %52

37:                                               ; preds = %33
  %38 = load [20 x i32]*, [20 x i32]** %5, align 8
  %39 = load i32, i32* %6, align 4
  %40 = sub nsw i32 %39, 1
  %41 = sext i32 %40 to i64
  %42 = getelementptr inbounds [20 x i32], [20 x i32]* %38, i64 %41
  %43 = load i32, i32* %7, align 4
  %44 = sext i32 %43 to i64
  %45 = getelementptr inbounds [20 x i32], [20 x i32]* %42, i64 0, i64 %44
  %46 = load i32, i32* %45, align 4
  %47 = icmp sge i32 %46, 1
  br i1 %47, label %48, label %51

48:                                               ; preds = %37
  %49 = load i32, i32* %9, align 4
  %50 = add nsw i32 %49, 1
  store i32 %50, i32* %9, align 4
  br label %51

51:                                               ; preds = %48, %37
  br label %52

52:                                               ; preds = %51, %33
  %53 = load i32, i32* %6, align 4
  %54 = sub nsw i32 %53, 1
  %55 = icmp sge i32 %54, 0
  br i1 %55, label %56, label %77

56:                                               ; preds = %52
  %57 = load i32, i32* %7, align 4
  %58 = add nsw i32 %57, 1
  %59 = load i32, i32* %8, align 4
  %60 = icmp slt i32 %58, %59
  br i1 %60, label %61, label %77

61:                                               ; preds = %56
  %62 = load [20 x i32]*, [20 x i32]** %5, align 8
  %63 = load i32, i32* %6, align 4
  %64 = sub nsw i32 %63, 1
  %65 = sext i32 %64 to i64
  %66 = getelementptr inbounds [20 x i32], [20 x i32]* %62, i64 %65
  %67 = load i32, i32* %7, align 4
  %68 = add nsw i32 %67, 1
  %69 = sext i32 %68 to i64
  %70 = getelementptr inbounds [20 x i32], [20 x i32]* %66, i64 0, i64 %69
  %71 = load i32, i32* %70, align 4
  %72 = icmp sge i32 %71, 1
  br i1 %72, label %73, label %76

73:                                               ; preds = %61
  %74 = load i32, i32* %9, align 4
  %75 = add nsw i32 %74, 1
  store i32 %75, i32* %9, align 4
  br label %76

76:                                               ; preds = %73, %61
  br label %77

77:                                               ; preds = %76, %56, %52
  %78 = load i32, i32* %7, align 4
  %79 = sub nsw i32 %78, 1
  %80 = icmp sge i32 %79, 0
  br i1 %80, label %81, label %96

81:                                               ; preds = %77
  %82 = load [20 x i32]*, [20 x i32]** %5, align 8
  %83 = load i32, i32* %6, align 4
  %84 = sext i32 %83 to i64
  %85 = getelementptr inbounds [20 x i32], [20 x i32]* %82, i64 %84
  %86 = load i32, i32* %7, align 4
  %87 = sub nsw i32 %86, 1
  %88 = sext i32 %87 to i64
  %89 = getelementptr inbounds [20 x i32], [20 x i32]* %85, i64 0, i64 %88
  %90 = load i32, i32* %89, align 4
  %91 = icmp sge i32 %90, 1
  br i1 %91, label %92, label %95

92:                                               ; preds = %81
  %93 = load i32, i32* %9, align 4
  %94 = add nsw i32 %93, 1
  store i32 %94, i32* %9, align 4
  br label %95

95:                                               ; preds = %92, %81
  br label %96

96:                                               ; preds = %95, %77
  %97 = load i32, i32* %7, align 4
  %98 = add nsw i32 %97, 1
  %99 = load i32, i32* %8, align 4
  %100 = icmp slt i32 %98, %99
  br i1 %100, label %101, label %116

101:                                              ; preds = %96
  %102 = load [20 x i32]*, [20 x i32]** %5, align 8
  %103 = load i32, i32* %6, align 4
  %104 = sext i32 %103 to i64
  %105 = getelementptr inbounds [20 x i32], [20 x i32]* %102, i64 %104
  %106 = load i32, i32* %7, align 4
  %107 = add nsw i32 %106, 1
  %108 = sext i32 %107 to i64
  %109 = getelementptr inbounds [20 x i32], [20 x i32]* %105, i64 0, i64 %108
  %110 = load i32, i32* %109, align 4
  %111 = icmp sge i32 %110, 1
  br i1 %111, label %112, label %115

112:                                              ; preds = %101
  %113 = load i32, i32* %9, align 4
  %114 = add nsw i32 %113, 1
  store i32 %114, i32* %9, align 4
  br label %115

115:                                              ; preds = %112, %101
  br label %116

116:                                              ; preds = %115, %96
  %117 = load i32, i32* %6, align 4
  %118 = add nsw i32 %117, 1
  %119 = load i32, i32* %8, align 4
  %120 = icmp slt i32 %118, %119
  br i1 %120, label %121, label %141

121:                                              ; preds = %116
  %122 = load i32, i32* %7, align 4
  %123 = sub nsw i32 %122, 1
  %124 = icmp sge i32 %123, 0
  br i1 %124, label %125, label %141

125:                                              ; preds = %121
  %126 = load [20 x i32]*, [20 x i32]** %5, align 8
  %127 = load i32, i32* %6, align 4
  %128 = add nsw i32 %127, 1
  %129 = sext i32 %128 to i64
  %130 = getelementptr inbounds [20 x i32], [20 x i32]* %126, i64 %129
  %131 = load i32, i32* %7, align 4
  %132 = sub nsw i32 %131, 1
  %133 = sext i32 %132 to i64
  %134 = getelementptr inbounds [20 x i32], [20 x i32]* %130, i64 0, i64 %133
  %135 = load i32, i32* %134, align 4
  %136 = icmp sge i32 %135, 1
  br i1 %136, label %137, label %140

137:                                              ; preds = %125
  %138 = load i32, i32* %9, align 4
  %139 = add nsw i32 %138, 1
  store i32 %139, i32* %9, align 4
  br label %140

140:                                              ; preds = %137, %125
  br label %141

141:                                              ; preds = %140, %121, %116
  %142 = load i32, i32* %6, align 4
  %143 = add nsw i32 %142, 1
  %144 = load i32, i32* %8, align 4
  %145 = icmp slt i32 %143, %144
  br i1 %145, label %146, label %161

146:                                              ; preds = %141
  %147 = load [20 x i32]*, [20 x i32]** %5, align 8
  %148 = load i32, i32* %6, align 4
  %149 = add nsw i32 %148, 1
  %150 = sext i32 %149 to i64
  %151 = getelementptr inbounds [20 x i32], [20 x i32]* %147, i64 %150
  %152 = load i32, i32* %7, align 4
  %153 = sext i32 %152 to i64
  %154 = getelementptr inbounds [20 x i32], [20 x i32]* %151, i64 0, i64 %153
  %155 = load i32, i32* %154, align 4
  %156 = icmp sge i32 %155, 1
  br i1 %156, label %157, label %160

157:                                              ; preds = %146
  %158 = load i32, i32* %9, align 4
  %159 = add nsw i32 %158, 1
  store i32 %159, i32* %9, align 4
  br label %160

160:                                              ; preds = %157, %146
  br label %161

161:                                              ; preds = %160, %141
  %162 = load i32, i32* %6, align 4
  %163 = add nsw i32 %162, 1
  %164 = load i32, i32* %8, align 4
  %165 = icmp slt i32 %163, %164
  br i1 %165, label %166, label %187

166:                                              ; preds = %161
  %167 = load i32, i32* %7, align 4
  %168 = add nsw i32 %167, 1
  %169 = load i32, i32* %8, align 4
  %170 = icmp slt i32 %168, %169
  br i1 %170, label %171, label %187

171:                                              ; preds = %166
  %172 = load [20 x i32]*, [20 x i32]** %5, align 8
  %173 = load i32, i32* %6, align 4
  %174 = add nsw i32 %173, 1
  %175 = sext i32 %174 to i64
  %176 = getelementptr inbounds [20 x i32], [20 x i32]* %172, i64 %175
  %177 = load i32, i32* %7, align 4
  %178 = add nsw i32 %177, 1
  %179 = sext i32 %178 to i64
  %180 = getelementptr inbounds [20 x i32], [20 x i32]* %176, i64 0, i64 %179
  %181 = load i32, i32* %180, align 4
  %182 = icmp sge i32 %181, 1
  br i1 %182, label %183, label %186

183:                                              ; preds = %171
  %184 = load i32, i32* %9, align 4
  %185 = add nsw i32 %184, 1
  store i32 %185, i32* %9, align 4
  br label %186

186:                                              ; preds = %183, %171
  br label %187

187:                                              ; preds = %186, %166, %161
  %188 = load i32, i32* %9, align 4
  ret i32 %188
}

declare i32 @usleep(i32 noundef) #2

attributes #0 = { noinline nounwind optnone uwtable "frame-pointer"="all" "min-legal-vector-width"="0" "no-trapping-math"="true" "stack-protector-buffer-size"="8" "target-cpu"="x86-64" "target-features"="+cx8,+fxsr,+mmx,+sse,+sse2,+x87" "tune-cpu"="generic" }
attributes #1 = { argmemonly nofree nounwind willreturn }
attributes #2 = { "frame-pointer"="all" "no-trapping-math"="true" "stack-protector-buffer-size"="8" "target-cpu"="x86-64" "target-features"="+cx8,+fxsr,+mmx,+sse,+sse2,+x87" "tune-cpu"="generic" }

!llvm.module.flags = !{!0, !1, !2, !3, !4}
!llvm.ident = !{!5}

!0 = !{i32 1, !"wchar_size", i32 4}
!1 = !{i32 7, !"PIC Level", i32 2}
!2 = !{i32 7, !"PIE Level", i32 2}
!3 = !{i32 7, !"uwtable", i32 1}
!4 = !{i32 7, !"frame-pointer", i32 2}
!5 = !{!"Ubuntu clang version 14.0.4-++20220524103116+29f1039a7285-1~exp1~20220524223156.141"}
!6 = distinct !{!6, !7}
!7 = !{!"llvm.loop.mustprogress"}
!8 = distinct !{!8, !7}
!9 = distinct !{!9, !7}
!10 = distinct !{!10, !7}
!11 = distinct !{!11, !7}
