BITS 64

%macro jz 2
    cmp %1,qword 0x0
    je  %2
%endmacro

jz rax,-0x21524111
jz rcx,-0x21524111
jz rdx,-0x21524111
jz rbx,-0x21524111
jz rsp,-0x21524111
jz rbp,-0x21524111
jz rsi,-0x21524111
jz rdi,-0x21524111
