  .text
  .intel_syntax noprefix
  .globl main
main:
.LBL0_0:
  push rbp
  mov rbp, rsp
  sub rsp, 16
  mov dword ptr [rbp-4], 2
  mov eax, dword ptr [rbp-4]
  mov ecx, eax
  add ecx, 1
  add eax, 2
  add ecx, eax
  mov eax, ecx
  add rsp, 16
  pop rbp
  ret 
