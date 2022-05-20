BITS 64

%macro jump_eq 2
    cmp %2,%1
    je  -0x21524111
%endmacro

%macro expand 1
    jump_eq %1,rax
    jump_eq %1,rcx
    jump_eq %1,rdx
    jump_eq %1,rbx
    jump_eq %1,rsp
    jump_eq %1,rbp
    jump_eq %1,rsi
    jump_eq %1,rdi
%endmacro

expand rax
expand rcx
expand rdx
expand rbx
expand rsp
expand rbp
expand rsi
expand rdi
