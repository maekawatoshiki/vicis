  .text
  .intel_syntax noprefix
.str:
  .string "hello world\00"
  .globl main
main:
.LBL0_0:
  push rbp
  mov rbp, rsp
  sub rsp, 16
  mov dword ptr [rbp-4], 0
  mov rdi, offset .str
  call puts
  mov eax, 0
  add rsp, 16
  pop rbp
  ret 
