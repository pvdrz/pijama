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
%endmacro

expand rax
expand rcx
expand rdx
expand rbx
expand rsp
expand rbp
expand rsi
expand rdi
