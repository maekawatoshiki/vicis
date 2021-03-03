  .text
  .intel_syntax noprefix
  .globl f
f:
.LBL0_0:
  push rbp
  mov rbp, rsp
  mov eax, 1
  pop rbp
  ret 
  .globl main
main:
.LBL1_0:
  push rbp
  mov rbp, rsp
  sub rsp, 16
  mov dword ptr [rbp-4], 0
  call f
  add rsp, 16
  pop rbp
  ret 
