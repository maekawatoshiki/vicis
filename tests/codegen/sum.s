  .text
  .intel_syntax noprefix
  .globl main
main:
.LBL0_0:
  push rbp
  mov rbp, rsp
  sub rsp, 16
  mov dword ptr [rbp-12], 0
  mov dword ptr [rbp-4], 0
  mov dword ptr [rbp-8], 1
  jmp .LBL0_1
.LBL0_1:
  mov eax, dword ptr [rbp-8]
  cmp eax, 10
  jle .LBL0_2
  jmp .LBL0_4
.LBL0_2:
  mov eax, dword ptr [rbp-8]
  mov ecx, dword ptr [rbp-4]
  add ecx, eax
  mov dword ptr [rbp-4], ecx
  jmp .LBL0_3
.LBL0_3:
  mov eax, dword ptr [rbp-8]
  add eax, 1
  mov dword ptr [rbp-8], eax
  jmp .LBL0_1
.LBL0_4:
  mov eax, dword ptr [rbp-4]
  add rsp, 16
  pop rbp
  ret 
