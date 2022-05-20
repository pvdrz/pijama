BITS 64

%macro expand 1
    add rax,%1
    add rcx,%1
    add rdx,%1
    add rbx,%1
    add rsp,%1
    add rbp,%1
    add rsi,%1
    add rdi,%1
%endmacro

expand rax
expand rcx
expand rdx
expand rbx
expand rsp
expand rbp
expand rsi
expand rdi
