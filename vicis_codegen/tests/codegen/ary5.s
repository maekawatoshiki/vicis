  .text
  .intel_syntax noprefix
  .globl main
main:
.LBL0_0:
  push rbp
  mov rbp, rsp
  sub rsp, 64
  mov dword ptr [rbp-56], 0
  mov dword ptr [rbp-4], 0
  mov dword ptr [rbp-52], 0
  jmp .LBL0_1
.LBL0_1:
  mov eax, dword ptr [rbp-52]
  cmp eax, 10
  jl .LBL0_2
  jmp .LBL0_4
.LBL0_2:
  mov eax, dword ptr [rbp-52]
  movsxd rcx, dword ptr [rbp-52]
  add eax, 1
  mov dword ptr [rbp-48+rcx*4], eax
  jmp .LBL0_3
.LBL0_3:
  mov eax, dword ptr [rbp-52]
  add eax, 1
  mov dword ptr [rbp-52], eax
  jmp .LBL0_1
.LBL0_4:
  mov dword ptr [rbp-8], 0
  jmp .LBL0_5
.LBL0_5:
  mov eax, dword ptr [rbp-8]
  cmp eax, 10
  jl .LBL0_6
  jmp .LBL0_8
.LBL0_6:
  movsxd rax, dword ptr [rbp-8]
  mov eax, dword ptr [rbp-48+rax*4]
  mov ecx, dword ptr [rbp-4]
  add ecx, eax
  mov dword ptr [rbp-4], ecx
  jmp .LBL0_7
.LBL0_7:
  mov eax, dword ptr [rbp-8]
  add eax, 1
  mov dword ptr [rbp-8], eax
  jmp .LBL0_5
.LBL0_8:
  mov eax, dword ptr [rbp-4]
  add rsp, 64
  pop rbp
  ret 
