
# Understand how to compile code to an executable file

Code generation will be restricted to Linux in `x86-64` for now. I need to
understand how to compile my own executable, given that I don't have the
knowledge to do this from scratch, I'm going to piggy back on top of `clang` to
achieve this. The plan would be to produce a `lib.o` object file containing a
`start` function. Then we write a simple `main.c` file that calls this extern
`start` function and prints its return value.

## Creating an object file from scratch

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
$ objdump -w -M intel -D lib.o
```

The `-w` flag tells `objdump` to use a wide output, `-M intel` is used to
enable the Intel syntax. The `-D` flag is used to disassemble the whole file.

From the output we can see that the assembly code for `start` is stored in a
`.text` section.

```objdump
Disassembly of section .text:

0000000000000000 <start>:
   0:    55                       push   rbp
   1:    48 89 e5                 mov    rbp,rsp
   4:    b8 0a 00 00 00           mov    eax,0xa
   9:    5d                       pop    rbp
   a:    c3                       ret
   b:    0f 1f 44 00 00           nop    DWORD PTR [rax+rax*1+0x0]
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
Additionally we know that this file is for the `x86-64` platform and that is
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

Just to be sure we got everything right we will add a second function to the
`lib.c` file and try to replicate the object file. Here's the new `.text`
section:
```objdump
Disassembly of section .text:

0000000000000000 <start>:
   0:    55                       push   rbp
   1:    48 89 e5                 mov    rbp,rsp
   4:    b8 0a 00 00 00           mov    eax,0xa
   9:    5d                       pop    rbp
   a:    c3                       ret
   b:    0f 1f 44 00 00           nop    DWORD PTR [rax+rax*1+0x0]

0000000000000010 <duplicate>:
  10:    55                       push   rbp
  11:    48 89 e5                 mov    rbp,rsp
  14:    89 7d fc                 mov    DWORD PTR [rbp-0x4],edi
  17:    8b 45 fc                 mov    eax,DWORD PTR [rbp-0x4]
  1a:    c1 e0 01                 shl    eax,0x1
  1d:    5d                       pop    rbp
  1e:    c3                       ret
```

And here is the symbol table:
```
   Num:    Value          Size Type    Bind   Vis      Ndx Name
     0: 0000000000000000     0 NOTYPE  LOCAL  DEFAULT  UND
     1: 0000000000000000     0 FILE    LOCAL  DEFAULT  ABS lib.c
     2: 0000000000000000     0 SECTION LOCAL  DEFAULT    2 .text
     3: 0000000000000000    11 FUNC    GLOBAL DEFAULT    2 start
     4: 0000000000000010    15 FUNC    GLOBAL DEFAULT    2 duplicate
```

As we can see the only different attributes are the value and the size. But we
can reuse what we already did and just append a second symbol.

## Generating machine code

Now that we are able to produce our own ELF files, we can start thinking about
writing an assembler to produce machine code.

### Understanding the `x86` instruction set

To do this, we need to understand the instruction format for `x86`. From the
Intel and AMD developer manuals for the `x86` architecture we can obtain the
format of the instructions:

```
┌──────────┬────────┬─────────┬─────┬──────────────┬───────────┐
│ Prefixes │ Opcode │ Mod R/M │ SIB │ Displacement │ Immediate │
└──────────┴────────┴────┬────┴──┬──┴──────────────┴───────────┘
                         │       │
                ┌────────┘       └──────────┐
                ▼                           ▼
    ┌─────┬────────────┬─────┐  ┌───────┬───────┬──────┐
    │ Mod │ Reg/Opcode │ R/M │  │ Scale │ Index │ Base │
    └─────┴────────────┴─────┘  └───────┴───────┴──────┘
```
Every instruction in the `x86` instruction set has this format. Some parts are
optional and others have variable length.

First, we have the `Prefixes`, for now the most important prefix is the `REX`
prefix which is used to enable 64-bit mode for certain instructions. This
prefix also interacts with the `Mod R/M` part in subtle ways.

The part that encodes which kind of instruction we are executing is the
`Opcode`, which can use anything from 1 to 3 bytes.

The `Mod R/M` and `SIB` bytes encode an operand of the instruction, this
operand resides in memory. These two bytes specify how to access this operand.
Some instructions don't require this.

Sometimes the `Mod R/M` bytes are followed by a `Displacement` which can use 1,
2 or 4 bytes.

Finally if an instruction has an immediate operand it is encoded in the
`Immediate` part which can use 1, 2 or 4 bytes.

As an example, let's take a look at the `mov` instruction inside the `start`
function at position `1`:
```objdump
0000000000000000 <start>:
   0:    55                       push   rbp
   1:    48 89 e5                 mov    rbp,rsp
   4:    b8 0a 00 00 00           mov    eax,0xa
   9:    5d                       pop    rbp
   a:    c3                       ret
   b:    0f 1f 44 00 00           nop    DWORD PTR [rax+rax*1+0x0]
```
The machine code is `48 89 e5` and the instruction is `mov rbp,rsp`.

Sadly, there are more than 30 different `mov` instructions in the [Intel
manual](https://www.felixcloutier.com/x86/mov). But based on the fact that both
operands are 64-bit registers and that the machine code has an `89` byte
somewhere. We can assume that the entry in the manual is this one:

```
┌───────────────┬───────────────┬───────┬─────────────┬─────────────────┬───────────────────┐
│    Opcode     │  Instruction  │ Op/En │ 64-Bit Mode │ Compat/Leg Mode │ Description       │
├───────────────┼───────────────┼───────┼─────────────┼─────────────────┼───────────────────┤
│ REX.W + 89 /r │ MOV r/m64,r64 │  MR   │    Valid    │       N.E.      │ Move r64 to m/r64 │
└───────────────┴───────────────┴───────┴─────────────┴─────────────────┴───────────────────┘
```

The `Opcode` column shows the form of the instruction. `REX.W` means that this
instruction uses the `REX` prefix to change its operand size or semantics (most
likely the former in this case). `+ 89` means that the `REX` prefix is followed
by an `89` byte. Finally `/r` means that the `Mod R/M` byte has a register
operand and a register or memory location operand.

The `Instruction` column is a bit clearer. Is saying that this instruction is
usually shown as `MOV`, followed by a 64-bit register or memory operand and
then a register operand. In our case, both operands are registers.

The `Op/En` or operand encoding column specifies how the operands are encoded.
Each instruction has a table in the Intel manual explaining the actual encoding
of the operands as this varies between instructions. For the `MOV` instruction
`MR` means that the first operand is encoded in the `r/m` field and that the
second operand is encoded in the `reg` field.

The `64-Bit Mode` and `Compat/Leg Mode` columns specify if this instruction is
valid in 64-bit mode and compatibility mode. We will choose instructions that
are valid in 64-bit mode and ignore the compatibility mode.

So as far as we know, this is the encoding of the instruction:
```
┌────────────┬───────────────┬────────────────┐
│ REX (0x48) │ Opcode (0x89) │ Mod R/M (0xe5) │
└────────────┴───────────────┴────────────────┘
```

Now we can try to reconstruct each part of the instruction:

The `REX` prefix is composed of three bits `W`, `R`, `B` and is written as a
single byte with the following binary format `0b0100WR0B`. From the `Opcode`
column we can infer that we only need to set the `W` bit. Meaning that the
first byte of the instruction should be `0b01001000` or `0x48`.

The second byte is easy as the `Opcode` column says it is `0x89`.

The third byte is a `Mod R/M` byte and it must encode the `rbp` and `rsp`
registers. We know that this `Mod R/M` is divided in three fields:
- The `mod` field which uses the bits 7-6.
- The `reg` field which uses the bits 5-3.
- The `r/m` field which uses the bits 2-0.

We gave this byte intervals "backwards" because we are in little-endian.

The `x86` manual has a table explaining how operands are encoded in these three
field and from the `Op/En` column we know that the first operand must be
encoded in the `r/m` field and the second in the `reg` field.

To encode the `bp` register as the first operand we set `mod` to `0b11` and
`r/m` to `0b101`. To encode the `sp` register as the second operand we set
`reg` to `0b100`. Meaning that the whole byte is `0b11100101` or `0xe5`.

So the machine code should be `48 89 e5`. Just like what we have in our
disassembled object file.

As we can see, the `x86` instruction set is very complex and writing code to
assemble every single instruction in the set would be an herculean task. But,
given that we are writing our own assembler, we can design our own instruction
set and encode it as valid `x86` instructions.

One reason to do this instead of just using a strict subset of the actual `x86`
instruction set is that it should be easier to port this assembler to other
platforms. If you're familiar with ARM, RISC-V or any other reduced instruction
set you'll notice that I took some inspiration from them.

### Writing a small instruction set

This will be our starting instruction set:

```
┌──────────────────────┬──────────────────────┬─────────────────────────────────────────────────────────────────────────────────┐
│ Name                 │ Instruction          │ Description                                                                     │
├──────────────────────┼──────────────────────┼─────────────────────────────────────────────────────────────────────────────────┤
│ Load Immediate       │ loadi imm64,reg      │ Load the imm64 value into reg.                                                  │
├──────────────────────┼──────────────────────┼─────────────────────────────────────────────────────────────────────────────────┤
│ Load Address         │ loada addr+imm32,reg │ Load the contents of addr + imm32 into reg.                                     │
├──────────────────────┼──────────────────────┼─────────────────────────────────────────────────────────────────────────────────┤
│ Store                │ store reg,addr+imm32 │ Store the contents of reg into addr + imm32.                                    │
├──────────────────────┼──────────────────────┼─────────────────────────────────────────────────────────────────────────────────┤
│ Move                 │ mov reg1,reg2        │ Move the contents of reg1 into reg2.                                            │
├──────────────────────┼──────────────────────┼─────────────────────────────────────────────────────────────────────────────────┤
│ Push                 │ push reg             │ Push the contents of reg into the stack.                                        │
├──────────────────────┼──────────────────────┼─────────────────────────────────────────────────────────────────────────────────┤
│ Pop                  │ pop reg              │ Pop a value from the stack and put it in reg.                                   │
├──────────────────────┼──────────────────────┼─────────────────────────────────────────────────────────────────────────────────┤
│ Add                  │ add reg1,reg2        │ Add the contents of reg1 to the contents of reg2.                               │
├──────────────────────┼──────────────────────┼─────────────────────────────────────────────────────────────────────────────────┤
│ Add Immediate        │ addi imm32,reg       │ Add the imm32 value to the contents of reg.                                     │
├──────────────────────┼──────────────────────┼─────────────────────────────────────────────────────────────────────────────────┤
│ Set If Less Than     │ slt reg1,reg2,reg3   │ Set reg3 to zero if the contents of reg1 are smaller than the contents of reg2. │
├──────────────────────┼──────────────────────┼─────────────────────────────────────────────────────────────────────────────────┤
│ Jump                 │ jmp addr             │ Jump to the value stored in addr.                                               │
├──────────────────────┼──────────────────────┼─────────────────────────────────────────────────────────────────────────────────┤
│ Jump If Zero         │ jz reg,imm32         │ Jump imm32 bytes if the contents of reg are zero.                               │
├──────────────────────┼──────────────────────┼─────────────────────────────────────────────────────────────────────────────────┤
│ Return               │ ret                  │ Transfer control to the address in the top of the stack.                        │
├──────────────────────┼──────────────────────┼─────────────────────────────────────────────────────────────────────────────────┤
│ Call                 │ call reg             │ Transfer control to the address contained in reg.                               │
└──────────────────────┴──────────────────────┴─────────────────────────────────────────────────────────────────────────────────┘
```

Before starting to generate machine code for these instructions we need to
clearly define their operands.

The easiest operand kind to understand are the immediates `imm32` and `imm64`
which are just constant signed integer values. We represent them with the `i32`
and `i64` types in Rust.

Then we have register operands which we denote by `reg`, `reg1`, `reg1`, `reg2`
and so on. The `x86` architecture has 16 general purpose registers in 64-bit
mode: `rax`, `rcx`, `rdx`, `rbx`, `rsp`, `rbp`, `rsi`, `rdi` `r8`, ..., `r14`
and `r15`, we will only use the first 8 for now. Additionally we have the `rip`
register which holds the instruction pointer. There are other specific purpose
registers that we will discuss if we need them.

Finally, we have addresses or `addr` which represent memory locations. For now,
we will say that base addresses are stored in a register, we will extend this
later. The base address can be modified by adding an offset to the base
address, this offset can only be an `imm32` and not an `imm64` (this is a
limitation of the `x86` instruction set).

Now we are ready to encode those instructions as valid `x86` machine code.
These are the instructions that we will use taken from the Intel's manual:

```
┌───────────────────┬──────────────────┬───────┬───────────────────────────────────────────────────────┐
│ Opcode            │ Instruction      │ Op/En │ Description                                           │
├───────────────────┼──────────────────┼───────┼───────────────────────────────────────────────────────┤
│ REX.W + B8+ rd io │ MOV r64, imm64   │ OI    │ Move imm64 to r64.                                    │
├───────────────────┼──────────────────┼───────┼───────────────────────────────────────────────────────┤
│ REX.W + 8B /r     │ MOV r64,r/m64    │ RM    │ Move r/m64 to r64.                                    │
├───────────────────┼──────────────────┼───────┼───────────────────────────────────────────────────────┤
│ REX.W + 89 /r     │ MOV r/m64,r64    │ MR    │ Move r64 to m/r64.                                    │
├───────────────────┼──────────────────┼───────┼───────────────────────────────────────────────────────┤
│ 50 + rd           │ PUSH r64         │ M     │ Push r/m64.                                           │
├───────────────────┼──────────────────┼───────┼───────────────────────────────────────────────────────┤
│ 58 + rd           │ POP r/m64        │ M     │ Pop top of stack into m64; increment stack pointer.   │
├───────────────────┼──────────────────┼───────┼───────────────────────────────────────────────────────┤
│ REX.W + 01 /r     │ ADD r/m64,r64    │ MR    │ Add r64 to r/m64.                                     │
├───────────────────┼──────────────────┼───────┼───────────────────────────────────────────────────────┤
│ REX.W + 81 /0 id  │ ADD r/m64,imm32  │ MI    │ Add imm32 sign-extended to 64-bits to r/m64.          │
├───────────────────┼──────────────────┼───────┼───────────────────────────────────────────────────────┤
│ REX.W + 05 id     │ ADD RAX,imm32    │ I     │ Add imm32 sign-extended to 64-bits to RAX.            │
├───────────────────┼──────────────────┼───────┼───────────────────────────────────────────────────────┤
│ REX.W + 0F 9C     │ SETL r/m8        │ M     │ Set byte if less.                                     │
├───────────────────┼──────────────────┼───────┼───────────────────────────────────────────────────────┤
│ REX.W + 3D id     │ CMP RAX,imm32    │ I     │ Compare imm32 sign-extended to 64-bits with RAX.      │
├───────────────────┼──────────────────┼───────┼───────────────────────────────────────────────────────┤
│ REX.W + 39 /r     │ CMP r/m64,r64    │ MR    │ Compare r64 with r/m64.                               │
├───────────────────┼──────────────────┼───────┼───────────────────────────────────────────────────────┤
│ REX.W + 81 /7 id  │ CMP r/m64,imm32  │ MI    │ Compare imm32 sign-extended to 64-bits with r/m64.    │
├───────────────────┼──────────────────┼───────┼───────────────────────────────────────────────────────┤
│ E9 cd             │ JMP rel32        │ M     │ Jump near, displacement relative to next instruction. │
├───────────────────┼──────────────────┼───────┼───────────────────────────────────────────────────────┤
│ 0F 84 cd          │ JE rel32         │ D     │ Jump near if equal.                                   │
├───────────────────┼──────────────────┼───────┼───────────────────────────────────────────────────────┤
│ C3                │ RET              │ ZO    │ Near return to calling procedure.                     │
├───────────────────┼──────────────────┼───────┼───────────────────────────────────────────────────────┤
│ FF /2             │ CALL r/m64       │ M     │ Call near, absolute indirect, address given in r/m64. │
└───────────────────┴──────────────────┴───────┴───────────────────────────────────────────────────────┘
```

all these instructions are valid in 64-bit mode. One important thing to notice
is that our syntax for two-operand instructions is different from Intel's syntax:

- In our syntax, the source operand precedes the destination operand, just like
  AT&T's syntax.

- In Intel's syntax, the destination operand precedes the source operand.

This means that our `addi 0x1,rax` instruction corresponds to the `ADD rax,0x1`
instruction using Intel's syntax.

You could also notice that all control-flow related instructions use the word
"near". This has to do with memory segmentation: near operations are
constrained to a single memory segment while far operations can move between
segments. Given that Linux has a flat-memory model and, more importantly, that
64-bit mode does not use segmentation, we won't be using "far" instructions at
all.

#### Load Immediate

To load an immedate to a register we will use the `MOV r64,imm64` instruction.
The first operand is encoded by adding an `rd` value representing the register
to the `0xB8` opcode and setting the `REX.B` bit to zero. The second operand is
encoded as trailing bytes.

The `rd` value changes from 0 to 7 for the first 8 general purpose registers in
the order they are written above.

To check that we generated the right machine code we will write an `x86`
assembly program that loads the `0xdeadbeefdeadbeef` integer to every single
one of the eight general purpose registers and assemble it using the netwide
assembler. Then we will compare the binary output of our assembler against
`nasm`'s.

This is the `x86` program:

```nasm
BITS 64
mov rax,-0x2152411021524111
mov rcx,-0x2152411021524111
mov rdx,-0x2152411021524111
mov rbx,-0x2152411021524111
mov rsp,-0x2152411021524111
mov rbp,-0x2152411021524111
mov rsi,-0x2152411021524111
mov rdi,-0x2152411021524111
```

We will store it in the `loadi.asm` file and assemble it in the `loadi.out`
file:
```bash
$ nasm -o loadi.out loadi.asm
```

Comparing both outputs is just as easy as calling `assert_eq!`, just to be sure
here is the `hexdump` of `loadi.out`:
```bash
$ hexdump -C loadi.out
00000000  48 b8 ef be ad de ef be  ad de 48 b9 ef be ad de  |H.........H.....|
00000010  ef be ad de 48 ba ef be  ad de ef be ad de 48 bb  |....H.........H.|
00000020  ef be ad de ef be ad de  48 bc ef be ad de ef be  |........H.......|
00000030  ad de 48 bd ef be ad de  ef be ad de 48 be ef be  |..H.........H...|
00000040  ad de ef be ad de 48 bf  ef be ad de ef be ad de  |......H.........|
00000050
```

Everything looks in order, so we are done with this instruction.

#### Load Address

To load something from an address to a register we will use the `MOV
r64,r/m64`, the only difference between this instruction and the `MOV
r/m64,r64` instruction is that the operands are encoded in the reverse order:
The first operand uses the `reg` field and the second operand uses the `r/m`
field.

The AMD manual does a great work explaining the logic behind the `Mod R/M` byte
encoding:

- The `mod` field is used encode the addressing mode of an operand. The
  direct-register mode is enabled by setting `mod` to `0b11`. Any value less
  than `0b11` uses an indirect-register mode. Direct in this context means that
  the contents of the register are the operand. Indirect means that the
  contents of the register are the address that stores the actual operand.

- The `reg` field is used to specify a register operand most of the time.

- The `r/m` field is used to specify a register operand if the `mod` field is
  `0b11`. If not it is used to encode a register or the presence of an `SIB`
  byte after the `Mod R/M` byte.

From this information we can build a pretty cool builder for `Mod R/M` bytes.
This has the advantage that we won't mistype or forget to set a field of the
byte that easily.

Going back to the `MOV r64,r/m64` instruction, we know that the first operand
goes in the `reg` field, and at least for this instruction it uses the same
encoding as the `+rd` value we saw in the previous `MOV r/m64,r64` instruction.
The actual encoding for this field changes according to the instruction.

The second operand is a bit more interesting to encode because addresses are
composed of a base address in a register and an offset. The operand has to use
the indirect-register mode because we want the address that the register holds.
But now we need to know how to encode the offset.

According to the AMD manual, setting the `mod` field to `0b01` or `0b10` allows
us to encode the offset in the `Displacement` field. Given that our offsets are
`imm32`, they fit perfectly using a 4-byte length displacement field.

Then we can use the `r/m` field to encode the actual register with almost same
encoding as the `reg` field. The only difference is that when we are not in
direct-register mode, the `0b100` value doesn't correspond to the `rsp`
register but instead enables the `SIB` mode.

However, we can use the `SIB` byte to specify the `rsp` register. As we saw
before, the `SIB` byte is composed of three fields:
- The `scale` field which uses the bits 7-6.
- The `index` field which uses the bits 5-3.
- The `base` field which uses the bits 2-0.

This byte specifies an effective address that is computed as `scale * index +
base` and given that the `Displacement` field is also included, the
displacement is added to this effective address.

We would like to encode the `rsp` register as the effective address. There are
some subtle differences but both `index` and `base` use the same encoding as
`reg` except in these cases:

- If `index` is set to `0b100`, then the actual index is zero.
- If `base` is set to `0b101` and `r/m` is `0b00`, then the actual base is
  zero.

The `scale` field can only encode 4 possible scales: 1, 2, 4 and 8.

Using this information we can build the `rsp` register setting the `index` to
zero, the `base` to the `rsp` register and the `scale` to 1 (we could use any
scale because it is multiplied by the index which is zero).

After solving this edge case, we can move on to testing the code generation for
this instruction. We will write a program that takes every pair of registers
(64 pairs in total) and loads into the second register whatever is stored in
the address computed by offsetting the contents of the first register by
`0xdeadbeef` bytes:
```nasm
BITS 64

%macro load 2
    mov %1,[%2-0x21524111]
%endmacro

%macro expand 1
    load %1,rax
    load %1,rcx
    load %1,rdx
    load %1,rbx
    load %1,rsp
    load %1,rbp
    load %1,rsi
    load %1,rdi
%endmacro

expand rax
expand rcx
expand rdx
expand rbx
expand rsp
expand rbp
expand rsi
expand rdi
```

We are using macros instead of writing the 64 pairs by hand to avoid possible errors.

#### Store

Store is the inverse of the load address instruction so it is reasonable to use
the instruction `MOV r/m64,r64`. Actually, this was the example instruction
that we used to understand how to emit machine code. So it should be pretty
straightforward.

We have to set the `REX.W` byte to enable 64-bit mode, the first operand is
encoded in the `r/m` field and that the second operand is encoded in the `reg`
field. This means that we can reuse most of the code that we did for the load
address instruction because the arguments are flipped.

We can test this instruction by doing the same we did for the `load`
instruction:
```nasm
BITS 64

%macro store 2
    mov [%1-0x21524111],%2
%endmacro

%macro expand 1
    store %1,rax
    store %1,rcx
    store %1,rdx
    store %1,rbx
    store %1,rsp
    store %1,rbp
    store %1,rsi
    store %1,rdi
%endmacro

expand rax
expand rcx
expand rdx
expand rbx
expand rsp
expand rbp
expand rsi
expand rdi
```
#### Move

We will encode this instruction by reusing the `MOV r/m64,r64` instruction with
one single difference: We will the `mod` field to `0x11` to enable the direct
adressing mode because we want to access the contents of the register instead
of interpreting them as an address.

We test this in the same as way we did with the `store` instruction:

```nasm
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
```

#### Push

The push instruction is the first instruction that we can basically copy from
the `x86` manual. We will use the `PUSH r64` instruction which encodes its
operand by adding its `rd` encoding to `0x50`.

We test this in the same as way we did with the `loadi` instruction:
```nasm
BITS 64
push rax
push rcx
push rdx
push rbx
push rsp
push rbp
push rsi
push rdi
```

#### Pop

We can also copy the pop instruction from the manual. The actual instruction is
`POP r64`, this instruction is encoded in a similar way to the `PUSH r64`
instruction but using `0x58` instead.

We test this instruction in the same way the `push` instruction:
```nasm
BITS 64
pop rax
pop rcx
pop rdx
pop rbx
pop rsp
pop rbp
pop rsi
pop rdi
```

#### Add

The instruction we will use for add is `ADD r/m64, r64` which encodes the first
operand in the `r/m` field and the second one in the `reg` field. We have to be
careful with the syntax order but other than that, emitting this instruction
should be straightforward.

We test this instruction in the same way as we did with the `load` instruction:

```nasm
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
 ```

#### Add Immediate

We will use the `ADD r/m64,imm32` instruction which adds the `imm32` value to
the contents of `r/m64`. This instruction is encoded by setting the `REX.W`
byte and setting the `reg` field to `0x0`. Additionally, the first operand is
encoded in the `r/m` field and the second operand is encoded as bytes after the
`mod r/m` byte.

However, our `addi` instruction has a particular case that can be written using
fewer bytes thanks to the `ADD RAX,imm32` instruction (yes, `x86` has a
specific "add to `rax`" instruction).

We test this instruction in the same way as we did with the `loadi`
instruction:

```nasm
BITS 64
add rax,-0x21524111
add rcx,-0x21524111
add rdx,-0x21524111
add rbx,-0x21524111
add rsp,-0x21524111
add rbp,-0x21524111
add rsi,-0x21524111
add rdi,-0x21524111
```

#### Set If Less Than

This is an interesting instruction because it does not map to a single
`x86` instruction. In `x86`, comparisons are done using with the `CMP`
instruction and its results are always stored in the special `RFLAGS` register.
This `RFLAGS` register is usually interpreted as a sequence of status flags
indicating the result of the comparison. Then, we can use the `SETL`
instruction to set the lowest byte of the target register to zero. However,
that doesn't clean the rest of the bytes of the register, which means that for
sanity we have to set the target register to zero before using `SETL`.

This means that `stl reg1,reg2,reg3` should be encoded as:

```nasm
cmp  reg1,reg2
mov  reg3,0x0
setl reg3
```

There are faster ways to set `reg3` to zero but using `mov` allow us to reuse
the code we wrote for Load Immediate. It is very important that we set `reg3`
to zero after we do the comparison, otherwise doing `slt a,b,a` would do
something like:
```nasm
mov  a,0x0
cmp  a,b
setl a
```

Overwriting the value of `a` before the comparison.

Now we can move on to the actual encoding. First, we will use the `CMP
r/m64,r64` which encodes the first operand in the `r/m` field and the second
operand in the `reg` field.

Then we will use the `SETL r/m8` instruction which encodes its operand in the
`r/m` field. The `reg` field can take any value but we will set it to zero.
This instruction requires the `REX` prefix to use the `rsp` `rbp`, `rsi` and
`rdi` registers so we have to take this special case into account.

We test this instruction with every possible register combination:
```nasm
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
```

See that the operand of `setl` is set to `al`, `cl`, `dl` and so on instead
of `rax`, `rcx`, `rdx` and so on because `setl` takes an 8-bit register, so we
must use the lowest-byte register counterpart to each 64-bit register.

#### Jump

We will use the `JMP rel32` instruction which encodes the operand as
displacement bytes. According to the Intel manual, this operand is relative to
the position of the instruction pointer after reading jump instruction (i.e.
the start of the next instruction).

However, `nasm` takes absolute positions and then encodes them as relative. For
example, the following program
```nasm
BITS 64
add rax,rcx
jmp 0x0
```
will jump back to the absolute position `0x0`, i.e. to `add rax,rcx`, instead
of doing nothing (jumping `0x0` bytes relative to the location of the
instruction pointer should be a no-op). We can see this by assembling this
program and disassembling it with `ndisasm -b64 -p intel <filename>`:

```nasm
00000000  4801C8            add rax,rcx
00000003  E9F8FFFFFF        jmp 0x0
```

From here we can see that `jmp 0x0` was encoded as `E9F8FFFFFF`. The first byte
we know it is the opcode `0xE9`. The remaining bytes, which are the
little-endian encoding of -8, should be the displacement, which is relative to
the start of the next instruction, or given that we don't have a next
instruction, the end of the file, which is exactly 8 (the `jmp` instruction
starts at byte 3 and it measures 5 bytes). So we are jumping -8 bytes relative
to 8, which is the same as jumping to absolute position 0.

We test this instruction by jumping with a relative offset of `0xdeadbeef`
twice to be sure we got the absolute to relative offset correction right:
```nasm
BITS 64
jmp -0x21524111
jmp -0x21524111
```

#### Conditional Jumps

We will start with the Jump If Zero instruction. This instruction performs a
conditional jump if the value contained in the second operand is zero. The
final position of the jump is in the first operand but this is relative to the
current location (in other words, the position of the instruction pointer after
reading this instruction).

In the same fashion as the Set If Less Than instruction, the first thing we
need to do is emit a compare instruction between our operand and zero. We will
use the `CMP r/m64,imm32` which encodes the first operand in the `r/m` field
and the second operand as displacement bytes. There is a special case for the
`RAX` register because we can use the `CMP RAX,imm32` instruction which is
shorter and encodes its operand as displacement bytes.

Then we emit a `JE rel32` or jump if equal instruction which encodes the
operand by appending it after the `0x84` byte.

In other words, we will encode our `jz reg,imm32` instruction as:
```nasm
cmp reg,0x0
je  imm32
```

We test this instruction in the same way as the other two-operand instructions:
```nasm BITS 64

%macro jump_eq 2
    cmp %1,%2
    je  -0x21524111
%endmacro

%macro expand 1
    jump_eq %1,rax
    jump_eq %1,rcx
    jump_eq %1,rdx
    jump_eq %1,rbx
    jump_eq %1,rsp
    jump_eq %1,rbp
    jump_eq %1,rsi
    jump_eq %1,rdi
%endmacro

expand rax
expand rcx
expand rdx
expand rbx
expand rsp
expand rbp
expand rsi
expand rdi
```

#### Return

After all this madness we go back to a simple instruction. We will use the near
return instruction `RET` which takes no operands.

Testing it is really simple because we only have to emit one instruction:
```nasm
BITS 64
ret
```

#### Call

For this last instruction we will use the near call instruction `CALL r/m64`
which encodes its operand in the `rm` field.

We test it in the same way as the other single-opreand instructions:
```nasm
BITS 64
call rax
call rcx
call rdx
call rbx
call rsp
call rbp
call rsi
call rdi
```

And we are done with our assembler!

### Emitting functions

Now that we can emit valid `x86` machine code, we should be able to write our
own functions inside it. To be able to do this we need to understand how
function calls and returns work. In other words, we need to understand the
calling convention. At least on Linux for `x86` machines, the calling
convention is defined by the System V ABI (this specification also defines ELF,
among other things).

This ABI specifies that parameters must be passed in the registers `rdi`,
`rsi`, `rdx`, `rcx`, `r8` and`r9`. If a function takes more parameters, these
are passed on the stack.

When the `call` instruction is executed, the location of the next instruction
is pushed into the stack. When `return` is executed, this location is popped
from the stack and the instruction pointer jumps to it.

Functions should leave the registers `rbx`, `rsp`, `rbp`, `r12`, `r13`, `r14`
and `r1` in the same state as they were before being called. The `rax`, `rdi`,
`rsi`, `rdx`, `rcx`, `r8`, `r9`, `r10` and `r11` registers can be modified
freely. The return value must be stored in `rax`.

Now we are ready to emit our own functions. We will start with the simple
`start` function that we had in our `lib.c` file:

```c
long start() {
    return 10;
}
```

From what we saw, the only thing we need to do is put the value `10` in the
`rax` register and then return, or in our assembly syntax:
```asm
loadi 0xa,rax
ret
```

We can do something more interesting like the `duplicate` function:
```c
long duplicate(long value) {
    return 2 * value;
}
```

Sadly we don't have a multiply instruction yet but we can do `value + value`
instead.

We know from the ABI that `rdi` has the first parameter of any function and
that the return value must be in the `rax` register. One way to write our
function would be

```asm
mov   rdi,rax
add   rdi,rax
ret
```

We first move `value` (which is stored in `rdi`) to `rax` and then add `value`
to `rax`.

If we link `main.c` against our object file and run it, we should get the same
output as with the `lib.c` library.

### Playing with Control Flow

We can use this `duplicate` function to test our different jump instructions
too. We could try to come up with an alternative implementation of `duplicate`
like this one:
```c
long duplicate(long value) {
    long output = 0;
    long i = 0;

    while i < value {
        output += 2;
        i += 1;
    }

    return output;
}
```

which should be equivalent to something like
```asm
loadi 0x0,rax    ; output = 0
loadi 0x0,rdx    ; i = 0

stl rdx,rdi,rcx  ; <CMP>: comp = i < output
jz  rcx,<END>    ; if comp is false, go to return

addi 0x2,rax     ; output += 2
addi 0x1,rdx     ; output += 1
jmp  <CMP>       ; go back to the comparison

ret              ; <END>: return
```

we need to figure out what are the values of `<END>` and `<CMP>`, to do this we
can set both of them to zero, assemble our code and then disassemble it with
`objdump`:
```objdump
0000000000000010 <duplicate>:
  10:   48 b8 00 00 00 00 00 00 00 00   movabs rax,0x0
  1a:   48 ba 00 00 00 00 00 00 00 00   movabs rdx,0x0
  24:   48 39 fa                cmp    rdx,rdi
  27:   48 b9 00 00 00 00 00 00 00 00   movabs rcx,0x0
  31:   0f 9c c1                setl   cl
  34:   48 81 f9 00 00 00 00    cmp    rcx,0x0
  3b:   0f 84 cf ff ff ff       je     10 <duplicate>
  41:   48 05 02 00 00 00       add    rax,0x2
  47:   48 81 c2 01 00 00 00    add    rdx,0x1
  4e:   e9 bd ff ff ff          jmp    10 <duplicate>
  53:   c3                      ret
```

From here we can appreciate that jump operands are taken by the assembler as
relative to the beginning of the symbol: `jmp 0x0` is encoded as a jump to
`0x10` because `duplicate` starts at `0x10`. This is just a convention in
reality because we know that jumps are always relative to the instruction
pointer.

We can also see that the first `cmp` instruction is at `0x24` (or `0x14`
relative to `duplicate`) and that the `ret` instruction is at `0x53` (or `0x43`
relative to `duplicate`). With this information we can finally write our
`duplicate` function as:
```asm
loadi 0x0,rax    ; output = 0
loadi 0x0,rdx    ; i = 0

stl rdx,rdi,rcx  ; comp = i < output
jz  rcx,0x43     ; if comp is false, go to return

addi 0x2,rax     ; output += 2
addi 0x1,rdx     ; output += 1
jmp  0x14        ; go back to the comparison

ret              ; return
```

We test this by compiling and linking our object file and `main.c` and then
running `a.out`, we should get the same output as before.

### Labels

This two-step process of first emitting code with placeholders for the jump
locations and then replacing them by the correct values is a bit tedious and
unreliable when done by hand. We need to remember which jumps should be updated
if we add, remove or change a instruction to our code. We can abstract it with
the concept of labels.

A label is an identifier used to reference a particular instruction, meaning
that we can use them instead of writing numeric positions:
```asm
      loadi 0x0,rax    ; output = 0
      loadi 0x0,rdx    ; i = 0

.cmp: stl rdx,rdi,rcx  ; comp = i < output
      jz  rcx,.end     ; if comp is false, go to return

      addi 0x2,rax     ; output += 2
      addi 0x1,rdx     ; output += 1
      jmp  .cmp        ; go back to the comparison

.end: ret              ; return
```

This means that our assembler should be able to know the location of an
instruction which has a label, even if the labels are defined "after" being
used. For example, the `.end` label is defined in the `ret` instruction which
appears later than its first use in the `jz` instruction.

We can solve this by mimicking the process we did manually: First we set a
placeholder value for all labels, and then we go back and patch those values
once we know the locations of all the labels. This process is know as
backpatching and it is wonderfully explained in [Crafting
Interpreters](https://craftinginterpreters.com/jumping-back-and-forth.html) by
Robert Nystrom.

# Writing an intermediate representation

Having our basic assembler working, we can move on to the next step and build
an intermediate representation that gives us a bit more structure.

This intermediate representation will be heavily inspired by [Rust's
MIR](https://rust-lang.github.io/rfcs/1211-mir.html)

The largest unit in MIR is a function. A function contains a collection of
basic blocks that can be thought as a directed graph.

```
BASIC_BLOCK ::= BB: STATEMENT* TERMINATOR
```

Each block has a label or identifier (`BB`) and is composed of zero or more
statements and one terminator.

The main difference between statements and terminators is that a statement
never affects the control-flow of the program, statements are always executed
sequentially and terminators alter the control-flow by calling other functions
or jumping to different blocks.

For now we will only define a single kind of statements, assignments:
```
STATEMENT ::= LOCAL = RVALUE
```

Each assignment has an `RVALUE` which represents a value to be computed and
then stored in a `LOCAL`.

We define two different kinds of rvalues, uses and binary operations:
```
RVALUE ::= USE OPERAND
         | OPERAND BINOP OPERAND

BINOP ::= + | <
```

Uses just yield the operand they contain and binary operations yield a new
value from two existing operands according to the binary operator they use, we
will only define the addition and the less than operators.

Operands are basically values. We have two kinds of them, constants and locals:
```
OPERAND ::= CONSTANT | LOCAL
```

We can think that constants are a sequence of bytes tagged with a type. For now
we will only consider 32-bit signed integers and booleans for now, which allows
us to restrict the byte sequence's length to four.

Which bring us back to locals, which are values local to a function, we can
think them as the state of the function and they are shared between blocks.
Following Rust's convention, the first local (usually denoted `_0`) stores the
value to be returned by the function. If our function has `N` arguments, those
are stored in locals `_1` to `_N`. Any additional locals hold intermediate
values introduced by either the programmer or the compiler.

Finally we have terminators:
```
TERMINATOR ::= JUMP BB
             | JUMP IF OPERAND THEN BB ELSE BB
             | RETURN
```

There are three kinds: unconditional jumps, conditional jumps based on the
truth value of an operand and returns.

To have a few examples we can reproduce the `start` and `duplicate` functions
we have been using:
```
// start
bb0: _0 = USE 10
     RETURN

// duplicate
bb0: _0 = USE 0
     _2 = USE 0
     JUMP bb1

bb1: _3 = _2 < _1
     JUMP IF _3 THEN bb2 ELSE bb3

bb2: _0 = _0 + 2
     _2 = _2 + 1
     JUMP bb1

bb3: RETURN
```

The `start` function is fairly simple, it puts the value `10` in the `_0` local
and returns. `duplicate` is a bit more complex, it uses two intermediate
locals: `_2` which holds the `i` variable, and `_3` which holds the result of
the comparison. However, it's structure is pretty similar to the assembly we
wrote before.

## Lowering to assembly

We could naively think that most of the elements of this representation
translate easily to our assembly. But we will discover quickly that there are
some details we need to take care of.

### Lowering constants

We will start by the easiest construct which is constants. As we said before,
constants are a sequence of bytes tagged with a type.

Regardless of the type we can interpret the bytes of the constant as an
`imm32`. To avoid any issue with booleans we will asume that zero represents
`false` and any other value represents `true`.

If we were to support constants like 64-bit integers we would have to do
several special cases because `x86` does not allow 64-bit immediates for most
instructions.

### Lowering locals

Locals are a bit more interesting because we could have more locals than
physical registers. The elegant solution to this issue is assigning each local
to a register temporarily, once the local is no longer used, the register is
freed and can be used for other local. However, it could happen that all
registers are being used while trying to assign a register to a local. In that
case, locals are "spilled" on to the stack. This whole process is known as
register allocation.

For now, we will do a very naive register allocation and assume that we always
have enough registers, later we will do something smarter. Even in this simple
scenario we must be careful about which registers can we assign to each local.

Thanks to the System V calling convention, we know that the return value must
go in the `rax` register, so we will assign `rax` to the `_0` local. We also
know that registers `rdi`, `rsi`, `rdx` and `rcx` hold the function's
parameters, at the same time the registers `rbx`, `rsp` and `rbp` must be left in
the same state as they were when calling the function (This is very important
because `rsp` holds the stack pointer, among other stuff).

So for our sake we won't touch the `rbx`, `rsp` and `rbp` registers for now.
Instead we will assign the `rax`, `rdi`, `rsi`, `rdx` and `rcx` registers to
the function's locals in that same order.

The limitations of this allocation strategy are that we can only lower
functions that have 4 or less parameters and that use 5 or less locals. Not
ideal, but it is a start.

### Lowering terminators

Given that terminators allow us to jump between basic blocks it is natural to
assign each basic block identifier a label. This is pretty straightforward as
we don't have any restriction in the number of labels or anything.

If the terminator is `JUMP BB`, we lower it directly to the `jmp` instruction,
mapping `BB` to its assigned label.

If the terminator is `JUMP IF COND THEN THEN_BB ELSE ELSE_BB`. First, the
`THEN_BB` and `ELSE_BB` identifiers are mapped into `then_label` and
`else_label` respectively. Then, we lower the `COND` operand as we explained in
the two previous sections. Given that we only have two kinds of operands, we
have two cases:

- If the operand was a local, it is mapped into a register `cond_reg`. Then we
  lower the terminator as following:
  ```asm
  jz  cond_reg,else_label
  jmp then_label
  ```

- If the operand was a constant, it is mapped into an immediate. We lower the
  terminator as `jmp else_label` if the immediate is zero or as `jmp
  then_label` otherwise.

If the terminator is `RETURN` we lower it directly to the `ret` instruction.

### Lowering statements

Finally, we have to figure out how to lower assignments. This is the trickiest
part because we need to consider every possible combination of operands for the
right hand side of the assignment. The left hand side is always a local, so we
can map it directly to a register `reg_lhs`.

If the rvalue is `USE operand`, we lower `operand` and operate based on the
lowering output:

- If we get a register `reg_op`, then we are moving the value stored in
  `reg_op` to `reg_lhs`. Then we lower this as `mov reg_op,reg_lhs`.

- If we get an immediate `imm_op`, then we lower this as `loadi
  imm_op,reg_lhs`.

If the `rvalue` is `op1 BINOP op2`, we lower `op1` and `op2` and operate based
in both outputs.

If we get two registers `reg_op1` and `reg_op2` respectively, we choose
instructions based on `BINOP`:

If it is `+` then we are storing the result of `reg_op1 + reg_op2` into
`reg_lhs`. Given that we cannot do this in a single instruction we must be
careful to not overwrite registers by accident. To handle this, we compare
`reg_lhs` against `reg_op1` and `reg_op2`:

- If `reg_lhs` is equal to `reg_op1`, then we are trying to do `reg_lhs =
  reg_lhs + reg_op2`. Which is the same as `reg_lhs += reg_op2`, meaning that
  we lower the whole statement as:
  ```asm
  add reg_op2,reg_lhs
  ```

- If `reg_lhs` is equal to `reg_op2`, we are trying to do `reg_lhs = reg_op1 +
  reg_lhs`. Assuming that addition is commutative (is it?) we can do `regh_lhs
  += reg_op1` and lower the statement as:
  ```asm
  add reg_op1,reg_lhs
  ```

- Otherwise, we know that `reg_lhs` is neither `reg_op1` nor `reg_op2`. Then we
  can overwrite `reg_lhs` with the contents of `reg_op1` and then add
  `reg_op2`:
  ```asm
  mov reg_op1,reg_lhs
  add reg_op2,reg_lhs
  ```

If `BINOP` is `<`, we can use the `slt` instruction directly: `slt
reg_op1,reg_op2,reg_lhs`.

We have other three cases to handle but we can easily choose instructions for
all of them with the same strategy we used in this first case, except for when
`BINOP` is `<` and one of the operands is lowered as an immediate because we
don't have a "Set If Less Than Immediate" instruction in our assembly. We will
leave a `todo!()` for those cases and go back to them later.

If we did everything correctly, we can lower the MIR for the `duplicate`
function and emit the following code:

```objdump
0000000000000010 <duplicate>:
  10:   48 b8 00 00 00 00 00 00 00 00   movabs rax,0x0
  1a:   48 be 00 00 00 00 00 00 00 00   movabs rsi,0x0
  24:   e9 00 00 00 00          jmp    29 <duplicate+0x19>
  29:   48 39 fe                cmp    rsi,rdi
  2c:   48 ba 00 00 00 00 00 00 00 00   movabs rdx,0x0
  36:   0f 9c c2                setl   dl
  39:   48 81 fa 00 00 00 00    cmp    rdx,0x0
  40:   0f 84 17 00 00 00       je     5d <duplicate+0x4d>
  46:   e9 00 00 00 00          jmp    4b <duplicate+0x3b>
  4b:   48 05 02 00 00 00       add    rax,0x2
  51:   48 81 c6 01 00 00 00    add    rsi,0x1
  58:   e9 cc ff ff ff          jmp    29 <duplicate+0x19>
  5d:   c3                      ret
```

It isn't the same assembly we wrote by hand before. The locations `24` and `46`
have `jmp` instructions that jump to the next instruction which is semantically
right but it is a bit wasteful. Nevertheless, if we link and run our main
program we will get the same output as before.

# Optimizing Code Generation

At this point you should have noticed that even though our code generation
works, the generated code could be improved by either removing instructions or
changing them by other instructions.

This means that we should try to first generate the instructions we are trying
to encode in a first stage and then emit machine code based on those
instructions in a second pass.

Another advantage of doing this is that it allows us to make our assembly
portable. The only platform specific part of it are the registers, so that
means that we can make our instructions generic over the type of registers we
are going to use.

## Removing Unnecessary Jumps

As we saw before, our MIR lowering stage introduces some instructions that jump
to the immediately next instruction. Given that we are able to manipulate
instructions before emmiting them, we can find such instructions by searching
every jump instruction whose target is a the label of the next instruction.

Even though it would be tempting to delete such instructions figuring out what
to do with the label of the instruction to be deleted is not trivial: We could
try to assign this label to the next instruction but it already has another
label, meaning that we would have to search all occurences of either label and
replace it by the other.

Instead, we will intoduce a no-op instruction that is ignored when emitting
machine code. So for example:
```asm
     addi 0x1,rax
foo: jmp bar
bar: mov rax,rcx
```

Is optimized to:
```asm
     addi 0x1,rax
foo: nop
bar: mov rax,rcx
```

And then, when labels are solved `foo` and `bar` will point to the same
location.

If we did everything correctly, the generated code for `duplicate` should look
like:
```objdump
0000000000000010 <duplicate>:
  10:   48 b8 00 00 00 00 00 00 00 00   movabs rax,0x0
  1a:   48 be 00 00 00 00 00 00 00 00   movabs rsi,0x0
  24:   48 39 fe                cmp    rsi,rdi
  27:   48 ba 00 00 00 00 00 00 00 00   movabs rdx,0x0
  31:   0f 9c c2                setl   dl
  34:   48 81 fa 00 00 00 00    cmp    rdx,0x0
  3b:   0f 84 12 00 00 00       je     53 <duplicate+0x43>
  41:   48 05 02 00 00 00       add    rax,0x2
  47:   48 81 c6 01 00 00 00    add    rsi,0x1
  4e:   e9 d1 ff ff ff          jmp    24 <duplicate+0x14>
  53:   c3                      ret
```

One important aspect is that this optimization is independent of the platform.
Jumping to the next instruction is always the same as not doing anything.

## Better Ways to Load Immediates

Up until now we have encoded our `loadi imm64,reg` instruction as `mov
reg,imm64`. However, most of the times we are using it to set registers to
zero. Like at the beginning of `duplicate` or when encoding the `stl`
instruction.

In the good ol' days. It was a well known fact that a faster way to do `mov
reg,0x0` was using `xor reg,reg` instead. Even if we do not care about that,
the `xor` operation takes less bytes anyway. Sadly, calling `xor` changes the
`RFLAGS` register which means that we cannot use in between `cmp` and `setl`.

For now we will focus on the regular case of encoding `loadi 0x0,reg` and then
figure out how to optimize `stl`.

First, we need to go back to the manual and get the appropiate `xor`
instruction, we have two options:
```
┌───────────────┬────────────────┬───────┬────────────────┐
│ Opcode        │ Instruction    │ Op/En │ Description    │
├───────────────┼────────────────┼───────┼────────────────┤
│ REX.W + 31 /r │ XOR r/m64, r64 │ MR    │ r/m64 XOR r64. │
├───────────────┼────────────────┼───────┼────────────────┤
│ REX.W + 33 /r │ XOR r64, r/m64 │ RM    │ r64 XOR r/m64. │
└───────────────┴────────────────┴───────┴────────────────┘
```

Both use the same number of bytes but for some reason `nasm` favors the first
one so we will use that one so we can test our code against `nasm`. This
instruction encodes it's first operand in the `r/m` register and the second one
in the `reg` register. This is one of those happy cases where the order doesn't
actually matter as both operands are the same register.

After this optimization, the emmited code should be:
```objdump
0000000000000010 <duplicate>:
  10:   48 31 c0                xor    rax,rax
  13:   48 31 f6                xor    rsi,rsi
  16:   48 39 fe                cmp    rsi,rdi
  19:   48 ba 00 00 00 00 00 00 00 00   movabs rdx,0x0
  23:   0f 9c c2                setl   dl
  26:   48 81 fa 00 00 00 00    cmp    rdx,0x0
  2d:   0f 84 12 00 00 00       je     45 <duplicate+0x35>
  33:   48 05 02 00 00 00       add    rax,0x2
  39:   48 81 c6 01 00 00 00    add    rsi,0x1
  40:   e9 d1 ff ff ff          jmp    16 <duplicate+0x6>
  45:   c3                      ret
```

To optimize `slt reg1,reg2,reg3` we will do it by putting `mov` before `cmp` if
`reg3` is not equal to `reg1` or `reg2`.

```objdump
0000000000000010 <duplicate>:
  10:   48 31 c0                xor    rax,rax
  13:   48 31 f6                xor    rsi,rsi
  16:   48 31 d2                xor    rdx,rdx
  19:   48 39 fe                cmp    rsi,rdi
  1c:   0f 9c c2                setl   dl
  1f:   48 81 fa 00 00 00 00    cmp    rdx,0x0
  26:   0f 84 12 00 00 00       je     3e <duplicate+0x2e>
  2c:   48 05 02 00 00 00       add    rax,0x2
  32:   48 81 c6 01 00 00 00    add    rsi,0x1
  39:   e9 d8 ff ff ff          jmp    16 <duplicate+0x6>
  3e:   c3                      ret
```

## Reducing Operand Size

Up until now we have tried to restrict our operands to 64-bit registers and
either 32 or 64-bit immediates. However, `x86-64` allows us to use just the 32,
16 or 8 lowest bits of each register instead.

According to the Intel manual, when in 64-bits mode: "32-bit operands generate
a 32-bit result, zero-extended to a 64-bit result in the destination
general-purpose register". This means that we can shave off some bytes by
checking if our immediates fit in 32 bits and then remove the `REX` prefix from
some instructions.

This is not the case for 16 and 8 bits operands: "8-bit and 16-bit operands
generate an 8-bit or 16-bit result. The upper 56 bits or 48 bits (respectively)
of the destination general-purpose register are not modified by the operation".
This is the reason why we had to clear the destination register for the `slt`
instruction.

Then, we can modify the code generation for `loadi` and check if the leading
32-bits of an integer are zero, if that is the case, we use the `MOV r32,
imm32` instruction instead. This instruction encoding is similar to `MOV
r64,imm64`, the only differences are that it does not use the `REX` prefix and
it uses 4 displacement bytes instead of 8.

Using this same idea, we can use `XOR r32,r32` to encode `loadi 0x0,reg` as the
upper part of the register will be overwritten with zeros.

We can also reduce the operand size for the `addi` instruction if the immediate
operand fits in a single byte by replacing `ADD r/m64,imm32` by the `ADD
r/m64,imm8` instruction which uses the opcode  `0x83` and a single displacement
byte instead.

After all these optimizations, our `duplicate` function should look like this:

```objdump
0000000000000010 <duplicate>:
  10:   31 c0                   xor    eax,eax
  12:   31 f6                   xor    esi,esi
  14:   31 d2                   xor    edx,edx
  16:   48 39 fe                cmp    rsi,rdi
  19:   0f 9c c2                setl   dl
  1c:   48 81 fa 00 00 00 00    cmp    rdx,0x0
  23:   0f 84 0d 00 00 00       je     36 <duplicate+0x26>
  29:   48 83 c0 02             add    rax,0x2
  2d:   48 83 c6 01             add    rsi,0x1
  31:   e9 de ff ff ff          jmp    14 <duplicate+0x4>
  36:   c3                      ret
```

When we took the MIR for `duplicate` and lowered it, the generated machine code
used 93 bytes, now we have reduced it to 54. However, it is important to
remember that this is a very particular case as most optimizations were
inspired by this function.
