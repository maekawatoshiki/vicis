  .text
  .intel_syntax noprefix
  .globl main
main:
.LBL0_0:
  push rbp
  mov rbp, rsp
  sub rsp, 16
  mov dword ptr [rbp-16], 0
  mov dword ptr [rbp-12], 0
  movsxd rax, dword ptr [rbp-12]
  mov dword ptr [rbp-8+rax*4], 1
  mov dword ptr [rbp-12], 1
  movsxd rax, dword ptr [rbp-12]
  mov dword ptr [rbp-8+rax*4], 2
  mov eax, 0
  add rsp, 16
  pop rbp
  ret 
