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
    slt %1,%2,r8,r8d,r8b
    slt %1,%2,r9,r9d,r9b
    slt %1,%2,r10,r10d,r10b
    slt %1,%2,r11,r11d,r11b
    slt %1,%2,r12,r12d,r12b
    slt %1,%2,r13,r13d,r13b
    slt %1,%2,r14,r14d,r14b
    slt %1,%2,r15,r15d,r15b
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
    slt2 %1,r8
    slt2 %1,r9
    slt2 %1,r10
    slt2 %1,r11
    slt2 %1,r12
    slt2 %1,r13
    slt2 %1,r14
    slt2 %1,r15
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
