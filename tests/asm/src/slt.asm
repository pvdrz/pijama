BITS 64

%macro slt 5
    %if %1 = %3
        cmp  %1,%2
        mov  %4,dword 0x0
    %elif %2 = %3
        cmp  %1,%2
        mov  %4,dword 0x0
    %else
        xor %4,%4
        cmp %1,%2
    %endif
    setl %5
%endmacro

%macro slt2 2
    slt %1,%2,rax,eax,al
    slt %1,%2,rcx,ecx,cl
    slt %1,%2,rdx,edx,dl
    slt %1,%2,rbx,ebx,bl
    slt %1,%2,rsp,esp,spl
    slt %1,%2,rbp,ebp,bpl
    slt %1,%2,rsi,esi,sil
    slt %1,%2,rdi,edi,dil
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


