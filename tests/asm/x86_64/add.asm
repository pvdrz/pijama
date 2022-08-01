BITS 64

%macro expand 16
  %assign i 0
  %rep %0
    %rotate i
    %define reg %1
    %rotate -i

    %rep %0
      add %1,reg
      %rotate 1
    %endrep

    %assign i i+1
  %endrep
%endmacro

expand rax,rcx,rdx,rbx,rsp,rbp,rsi,rdi,r8,r9,r10,r11,r12,r13,r14,r15
