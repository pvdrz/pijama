BITS 64

%macro jump_lt 2
    cmp %2,%1
    jl  -0x21524111
%endmacro

%macro expand 1
    jump_lt %1,rax
    jump_lt %1,rcx
    jump_lt %1,rdx
    jump_lt %1,rbx
    jump_lt %1,rsp
    jump_lt %1,rbp
    jump_lt %1,rsi
    jump_lt %1,rdi
%endmacro

expand rax
expand rcx
expand rdx
expand rbx
expand rsp
expand rbp
expand rsi
expand rdi
