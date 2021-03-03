  .text
  .intel_syntax noprefix
  .globl fibo
fibo:
.LBL0_0:
  push rbp
  mov rbp, rsp
  sub rsp, 16
  mov eax, edi
  mov dword ptr [rbp-12], eax
  mov eax, dword ptr [rbp-12]
  cmp eax, 2
  jle .LBL0_1
  jmp .LBL0_2
.LBL0_1:
  mov dword ptr [rbp-4], 1
  jmp .LBL0_3
.LBL0_2:
  mov eax, dword ptr [rbp-12]
  sub eax, 1
  mov edi, eax
  call fibo
  mov dword ptr [rbp-8], eax
  mov eax, dword ptr [rbp-12]
  sub eax, 2
  mov edi, eax
  call fibo
  mov ecx, dword ptr [rbp-8]
  add ecx, eax
  mov dword ptr [rbp-4], ecx
  jmp .LBL0_3
.LBL0_3:
  mov eax, dword ptr [rbp-4]
  add rsp, 16
  pop rbp
  ret 
  .globl main
main:
.LBL1_0:
  push rbp
  mov rbp, rsp
  sub rsp, 16
  mov dword ptr [rbp-4], 0
  mov edi, 10
  call fibo
  add rsp, 16
  pop rbp
  ret 
