  .text
  .intel_syntax noprefix
  .globl main
main:
.LBL0_0:
  push rbp
  mov rbp, rsp
  sub rsp, 32
  mov dword ptr [rbp-20], 0
  mov dword ptr [rbp-16], 0
  mov dword ptr [rbp-12], 1
  mov dword ptr [rbp-8], 2
  mov dword ptr [rbp-4], 3
  mov eax, 0
  add rsp, 32
  pop rbp
  ret 
