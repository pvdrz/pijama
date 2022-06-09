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
    add r8,%1
    add r9,%1
    add r10,%1
    add r11,%1
    add r12,%1
    add r13,%1
    add r14,%1
    add r15,%1
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
