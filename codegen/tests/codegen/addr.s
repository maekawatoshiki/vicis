  .text
  .intel_syntax noprefix
  .text
  .globl main
main:
.LBL0_0:
  push rbp
  mov rbp, rsp
  sub rsp, 16
  mov dword ptr [rbp-16], 0
  mov dword ptr [rbp-12], 42
  lea rax, [rbp-12]
  mov qword ptr [rbp-8], rax
  mov rax, qword ptr [rbp-8]
  mov eax, dword ptr [rax]
  add rsp, 16
  pop rbp
  ret 
