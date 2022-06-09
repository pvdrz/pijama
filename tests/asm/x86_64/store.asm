BITS 64

%macro store 2
    mov [%1-0x21524111],%2
%endmacro

%macro expand 1
    store %1,rax
    store %1,rcx
    store %1,rdx
    store %1,rbx
    store %1,rsp
    store %1,rbp
    store %1,rsi
    store %1,rdi
    store %1,r8
    store %1,r9
    store %1,r10
    store %1,r11
    store %1,r12
    store %1,r13
    store %1,r14
    store %1,r15
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
