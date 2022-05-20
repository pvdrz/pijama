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
%endmacro

expand rax
expand rcx
expand rdx
expand rbx
expand rsp
expand rbp
expand rsi
expand rdi
