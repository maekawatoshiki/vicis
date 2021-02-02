  .text
  .intel_syntax noprefix
  .globl main
main:
.LBL0:
  push rbp
  mov rbp, rsp
  sub rsp, 16
  mov dword ptr [rbp-4], 2
  mov eax, dword ptr [rbp-4]
  cmp eax, 2
  je .LBL1
  jmp .LBL2
.LBL1:
  mov eax, 1
  add rsp, 16
  pop rbp
  ret 
.LBL2:
  mov eax, 2
  add rsp, 16
  pop rbp
  ret 
