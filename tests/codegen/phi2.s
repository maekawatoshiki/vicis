  .text
  .intel_syntax noprefix
  .globl main
main:
.LBL0:
  push rbp
  mov rbp, rsp
  mov eax, 0
  mov ecx, 1
  jmp .LBL1
.LBL1:
  cmp ecx, 10
  jle .LBL2
  jmp .LBL4
.LBL2:
  add eax, ecx
  jmp .LBL3
.LBL3:
  add ecx, 1
  jmp .LBL1
.LBL4:
  pop rbp
  ret 
