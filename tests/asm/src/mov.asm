BITS 64

%macro expand 1
    mov rax,%1
    mov rcx,%1
    mov rdx,%1
    mov rbx,%1
    mov rsp,%1
    mov rbp,%1
    mov rsi,%1
    mov rdi,%1
%endmacro

expand rax
expand rcx
expand rdx
expand rbx
expand rsp
expand rbp
expand rsi
expand rdi
