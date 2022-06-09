BITS 64

%macro load 2
    mov %2,[%1-0x21524111]
%endmacro

%macro expand 1
    load %1,rax
    load %1,rcx
    load %1,rdx
    load %1,rbx
    load %1,rsp
    load %1,rbp
    load %1,rsi
    load %1,rdi
    load %1,r8
    load %1,r9
    load %1,r10
    load %1,r11
    load %1,r12
    load %1,r13
    load %1,r14
    load %1,r15
%endmacro

expand rax
expand rcx
expand rdx
expand rbx
expand rsp
expand rbp
expand rsi
expand rdi
expand r8
expand r9
expand r10
expand r11
expand r12
expand r13
expand r14
expand r15
