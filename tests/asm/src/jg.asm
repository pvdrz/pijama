BITS 64

%macro jump_gt 2
    cmp %2,%1
    jg  -0x21524111
%endmacro

%macro expand 1
    jump_gt %1,rax
    jump_gt %1,rcx
    jump_gt %1,rdx
    jump_gt %1,rbx
    jump_gt %1,rsp
    jump_gt %1,rbp
    jump_gt %1,rsi
    jump_gt %1,rdi
%endmacro

expand rax
expand rcx
expand rdx
expand rbx
expand rsp
expand rbp
expand rsi
expand rdi
