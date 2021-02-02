  .text
  .intel_syntax noprefix
  .globl main
main:
.LBL0:
  push rbp
  mov rbp, rsp
  sub rsp, 16
  mov dword ptr [rbp-12], 0
  mov dword ptr [rbp-4], 0
  mov dword ptr [rbp-8], 1
  jmp .LBL1
.LBL1:
  mov eax, dword ptr [rbp-8]
  cmp eax, 10
  jle .LBL2
  jmp .LBL4
.LBL2:
  mov eax, dword ptr [rbp-8]
  mov ecx, dword ptr [rbp-4]
  add ecx, eax
  mov dword ptr [rbp-4], ecx
  jmp .LBL3
.LBL3:
  mov eax, dword ptr [rbp-8]
  add eax, 1
  mov dword ptr [rbp-8], eax
  jmp .LBL1
.LBL4:
  mov eax, dword ptr [rbp-4]
  add rsp, 16
  pop rbp
  ret 
