  .text
  .intel_syntax noprefix
  .globl main
main:
.LBL0_0:
  push rbp
  mov rbp, rsp
  sub rsp, 16
  mov dword ptr [rbp-4], 1
  mov eax, dword ptr [rbp-4]
  cmp eax, 0
  je .LBL0_1
  jmp .LBL0_2
.LBL0_1:
  mov eax, 1
  jmp .LBL0_3
.LBL0_2:
  mov eax, 2
  jmp .LBL0_3
.LBL0_3:
  add rsp, 16
  pop rbp
  ret 
