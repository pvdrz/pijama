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
    mov r8,%1
    mov r9,%1
    mov r10,%1
    mov r11,%1
    mov r12,%1
    mov r13,%1
    mov r14,%1
    mov r15,%1
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
