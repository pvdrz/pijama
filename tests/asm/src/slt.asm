BITS 64

%macro slt 4
    cmp  %1,%2
    mov  %3,qword 0x0
    setl %4
%endmacro

%macro slt2 2
    slt %1,%2,rax,al
    slt %1,%2,rcx,cl
    slt %1,%2,rdx,dl
    slt %1,%2,rbx,bl
    slt %1,%2,rsp,spl
    slt %1,%2,rbp,bpl
    slt %1,%2,rsi,sil
    slt %1,%2,rdi,dil
%endmacro

%macro expand 1
    slt2 %1,rax
    slt2 %1,rcx
    slt2 %1,rdx
    slt2 %1,rbx
    slt2 %1,rsp
    slt2 %1,rbp
    slt2 %1,rsi
    slt2 %1,rdi
%endmacro

expand rax
expand rcx
expand rdx
expand rbx
expand rsp
expand rbp
expand rsi
expand rdi
