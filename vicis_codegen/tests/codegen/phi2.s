  .text
  .intel_syntax noprefix
  .globl main
main:
.LBL0_0:
  push rbp
  mov rbp, rsp
  mov eax, 0
  mov ecx, 1
  jmp .LBL0_1
.LBL0_1:
  cmp ecx, 10
  jle .LBL0_2
  jmp .LBL0_4
.LBL0_2:
  add eax, ecx
  jmp .LBL0_3
.LBL0_3:
  add ecx, 1
  jmp .LBL0_1
.LBL0_4:
  pop rbp
  ret 
