  .text
  .intel_syntax noprefix
  .globl f
f:
.LBL0:
  push rbp
  mov rbp, rsp
  mov eax, edi
  pop rbp
  ret 
  .globl main
main:
.LBL0:
  push rbp
  mov rbp, rsp
  mov edi, 1
  call f
  pop rbp
  ret 
