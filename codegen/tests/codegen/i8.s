  .text
  .intel_syntax noprefix
  .text
  .globl main
main:
.LBL0_0:
  push rbp
  mov rbp, rsp
  sub rsp, 16
  mov dword ptr [rbp-5], 0
  mov byte ptr [rbp-1], 1
  mov eax, 0
  add rsp, 16
  pop rbp
  ret 
