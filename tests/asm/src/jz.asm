BITS 64

%macro jz 1
    cmp %1,qword 0x0
    je  0x0
%endmacro

jz rax
jz rcx
jz rdx
jz rbx
jz rsp
jz rbp
jz rsi
jz rdi
jz r8
jz r9
jz r10
jz r11
jz r12
jz r13
jz r14
jz r15
