  .text
  .intel_syntax noprefix
  .globl main
main:
.LBL0:
  push rbp
  mov rbp, rsp
  sub rsp, 64
  mov dword ptr [rbp-56], 0
  mov dword ptr [rbp-4], 0
  mov dword ptr [rbp-52], 0
  jmp .LBL1
.LBL1:
  mov eax, dword ptr [rbp-52]
  cmp eax, 10
  jl .LBL2
  jmp .LBL4
.LBL2:
  mov eax, dword ptr [rbp-52]
  mov ecx, dword ptr [rbp-52]
  movsxd rcx, ecx
  add eax, 1
  mov dword ptr [rbp-48+rcx*4], eax
  jmp .LBL3
.LBL3:
  mov eax, dword ptr [rbp-52]
  add eax, 1
  mov dword ptr [rbp-52], eax
  jmp .LBL1
.LBL4:
  mov dword ptr [rbp-8], 0
  jmp .LBL5
.LBL5:
  mov eax, dword ptr [rbp-8]
  cmp eax, 10
  jl .LBL6
  jmp .LBL8
.LBL6:
  mov eax, dword ptr [rbp-8]
  movsxd rax, eax
  mov eax, dword ptr [rbp-48+rax*4]
  mov ecx, dword ptr [rbp-4]
  add ecx, eax
  mov dword ptr [rbp-4], ecx
  jmp .LBL7
.LBL7:
  mov eax, dword ptr [rbp-8]
  add eax, 1
  mov dword ptr [rbp-8], eax
  jmp .LBL5
.LBL8:
  mov eax, dword ptr [rbp-4]
  add rsp, 64
  pop rbp
  ret 
