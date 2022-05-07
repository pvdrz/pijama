
# Understand how to compile code to an executable file

Code generation will be restricted to Linux in `x86_64` for now. I need to
understand how to compile my own executable, given that I don't have the
knowledge to do this from scratch, I'm going to piggy back on top of `clang` to
achieve this. The plan would be to produce a `lib.o` object file containing a
`start` function. Then we write a simple `main.c` file that calls this extern
`start` function and prints its return value.

So to figure out how this `lib.o` should look like we will start by writing a
`lib.c` first and create an object file from it:
```bash
$ clang -c lib.c
```

This should create a `lib.o` object file. Then we can link `main.c` against it.
```bash
$ clang main.c lib.o
```

Now we should have a working `a.out` executable.

From here we can start inspecting the `lib.o` file. First we disassemble it:
```bash
$ objdump -D lib.o
```

The `-D` flag is used to disassemble the whole file.

From the output we can see that the assembly code for `start` is stored in a
`.text` section.

```
Disassembly of section .text:

0000000000000000 <start>:
   0:	55                   	push   %rbp
   1:	48 89 e5             	mov    %rsp,%rbp
   4:	b8 0a 00 00 00       	mov    $0xa,%eax
   9:	5d                   	pop    %rbp
   a:	c3                   	ret
```

Even better we even have the actual machine code for `start` on the second
column: `55 48 89 e5 b8 0a 00 00 00 5d c3`. Now we need to figure out how to
emit the rest of the object file and shove these bytes in the right place.

To figure out the remaining parts of the object file we could try to understand
what format uses:
```bash
$ file lib.o
lib.o: ELF 64-bit LSB relocatable, x86-64, version 1 (SYSV), not stripped
```

So, at least on Linux we use the Executable and Linkable Format or ELF.
Additionally we know that this file is for the `x86_64` platform and that is
encoding its binary data using least significant byte endianness, also known as
LSB or little-endian.

The only compiler backend written in Rust that I know of is `cranelift`. Which
has this `cranelift-object` crate to emit object files. This crate uses the
`object` crate to achieve this so we are going to use that.

Every ELF file is composed of an ELF header, a section header table and a
program header table.

We will ignore the ELF header as `object` manages that for us. We don't know
much about this format but we know that we need to create a `.text` section
with the function information so we should understand what a section is. We can
inspect the section headers of our object file
```bash
$ readelf -S lib.o
There are 9 section headers, starting at offset 0x178:

Section Headers:
  [Nr] Name              Type             Address           Offset
       Size              EntSize          Flags  Link  Info  Align
  [ 0]                   NULL             0000000000000000  00000000
       0000000000000000  0000000000000000           0     0     0
  [ 1] .strtab           STRTAB           0000000000000000  00000118
       0000000000000059  0000000000000000           0     0     1
  [ 2] .text             PROGBITS         0000000000000000  00000040
       000000000000000b  0000000000000000  AX       0     0     16
  [ 3] .comment          PROGBITS         0000000000000000  0000004b
       0000000000000016  0000000000000001  MS       0     0     1
  [ 4] .note.GNU-stack   PROGBITS         0000000000000000  00000061
       0000000000000000  0000000000000000           0     0     1
  [ 5] .eh_frame         X86_64_UNWIND    0000000000000000  00000068
       0000000000000038  0000000000000000   A       0     0     8
  [ 6] .rela.eh_frame    RELA             0000000000000000  00000100
       0000000000000018  0000000000000018           8     5     8
  [ 7] .llvm_addrsig     LOOS+0xfff4c03   0000000000000000  00000118
       0000000000000000  0000000000000000   E       8     0     1
  [ 8] .symtab           SYMTAB           0000000000000000  000000a0
       0000000000000060  0000000000000018           1     3     8
Key to Flags:
  W (write), A (alloc), X (execute), M (merge), S (strings), I (info),
  L (link order), O (extra OS processing required), G (group), T (TLS),
  C (compressed), x (unknown), o (OS specific), E (exclude),
  D (mbind), l (large), p (processor specific)
```

There we can see our `.text` section but we cannot see `start` which seems to
be some substructure inside the section. This is stored in the symbol table or
`.symtab` section:
```bash
$ readelf -s lib.o

Symbol table '.symtab' contains 4 entries:
   Num:    Value          Size Type    Bind   Vis      Ndx Name
     0: 0000000000000000     0 NOTYPE  LOCAL  DEFAULT  UND
     1: 0000000000000000     0 FILE    LOCAL  DEFAULT  ABS lib.c
     2: 0000000000000000     0 SECTION LOCAL  DEFAULT    2 .text
     3: 0000000000000000    11 FUNC    GLOBAL DEFAULT    2 start
```
there we can see the `start` symbol with some of its properties. To create a
symbol in `object` we need to provide its name, value and size which we already
have from this last output, and several other properties that we don't know
yet.

The first one is the kind of the symbol. From the `object` documentation we can
see that the kind text is used for executable code. Which is consistent with
the fact that this symbol is in the `.text` section too, so we will use that.

Then we need to know if the symbol should have weak binding or not. ELF has
three binding modes: Global, local and weak so we can assume that this symbol
does not have weak binding as the symbol table shows `GLOBAL` under the `Bind`
column.

We also need to know the section this symbol belongs to but we know that this
is the `.text` section.

Finally we need to know the symbol's flags. We don't know what those are but
`object` provides a struct with two fields: `st_info` and `st_other`. After a
quick googling session I found
[this](https://docs.oracle.com/cd/E19683-01/816-1386/chapter6-79797/index.html)
section in the Oracle's linker and libraries guide explaining both fields:

`st_info` contains the symbol's type and binding properties. Which we know are
`FUNC` and `GLOBAL` respectively from the symbol table. The global binding flag
is `1` and the function type flag is `2` and we can build the `st_info` value
as following: `(bind << 4) + (type & 0xf)`.

`st_other` contains the symbol's visibility. We know that our symbol has
`DEFAULT` visibility from the symbol table. The flag for default visibility is
`0` and we can build the `st_other` value as following: `vis & 0x3`. I wonder
why are there so many redundancies about binding in `object`.

With all this we can build our `start` symbol and add it to the `.text`
section. And we are done, we are able to produce a valid object file as a
substitute for our original `lib.o` file.
