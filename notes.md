
# Understand how to compile code to an executable file

Code generation will be restricted to Linux in `x86_64` for now. I need to
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
   0:	55                   	push   rbp
   1:	48 89 e5             	mov    rbp,rsp
   4:	b8 0a 00 00 00       	mov    eax,0xa
   9:	5d                   	pop    rbp
   a:	c3                   	ret
   b:	0f 1f 44 00 00       	nop    DWORD PTR [rax+rax*1+0x0]
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

Just to be sure we got everything right we will add a second function to the
`lib.c` file and try to replicate the object file. Here's the new `.text`
section:
```objdump
Disassembly of section .text:

0000000000000000 <start>:
   0:	55                   	push   rbp
   1:	48 89 e5             	mov    rbp,rsp
   4:	b8 0a 00 00 00       	mov    eax,0xa
   9:	5d                   	pop    rbp
   a:	c3                   	ret
   b:	0f 1f 44 00 00       	nop    DWORD PTR [rax+rax*1+0x0]

0000000000000010 <duplicate>:
  10:	55                   	push   rbp
  11:	48 89 e5             	mov    rbp,rsp
  14:	89 7d fc             	mov    DWORD PTR [rbp-0x4],edi
  17:	8b 45 fc             	mov    eax,DWORD PTR [rbp-0x4]
  1a:	c1 e0 01             	shl    eax,0x1
  1d:	5d                   	pop    rbp
  1e:	c3                   	ret
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

To do this, we need to understand the instruction format for `x86_64`. From the
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
   0:	55                   	push   rbp
   1:	48 89 e5             	mov    rbp,rsp
   4:	b8 0a 00 00 00       	mov    eax,0xa
   9:	5d                   	pop    rbp
   a:	c3                   	ret
   b:	0f 1f 44 00 00       	nop    DWORD PTR [rax+rax*1+0x0]
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
set and encode it as valid `x86-64` instructions.

One reason to do this instead of just using a strict subset of the actual `x86`
instruction set is that it should be easier to port this assembler to other
platforms. If you're familiar with ARM, RISC-V or any other reduced instruction
set you'll notice that I took some inspiration from them.

### Writing a small instruction set

This will be our starting instruction set:

```
┌──────────────────────┬──────────────────────┬────────────────────────────────────────────────────────────────────────────────────┐
│ Name                 │ Instruction          │ Description                                                                        │
├──────────────────────┼──────────────────────┼────────────────────────────────────────────────────────────────────────────────────┤
│ Load Immediate       │ loadi imm64,reg      │ Load the imm64 value into reg.                                                     │
├──────────────────────┼──────────────────────┼────────────────────────────────────────────────────────────────────────────────────┤
│ Load Address         │ loada addr+imm32,reg │ Load the contents of addr + imm32 into reg.                                        │
├──────────────────────┼──────────────────────┼────────────────────────────────────────────────────────────────────────────────────┤
│ Store                │ store reg,addr+imm32 │ Store the contents of reg into addr + imm32.                                       │
├──────────────────────┼──────────────────────┼────────────────────────────────────────────────────────────────────────────────────┤
│ Push                 │ push reg             │ Push the contents of reg into the stack.                                           │
├──────────────────────┼──────────────────────┼────────────────────────────────────────────────────────────────────────────────────┤
│ Pop                  │ pop reg              │ Pop a value from the stack and put it in reg.                                      │
├──────────────────────┼──────────────────────┼────────────────────────────────────────────────────────────────────────────────────┤
│ Add                  │ add reg1,reg2        │ Add the contents of reg1 to the contents of reg2.                                  │
├──────────────────────┼──────────────────────┼────────────────────────────────────────────────────────────────────────────────────┤
│ Add Immediate        │ addi imm32,reg       │ Add the imm32 value to the contents of reg.                                        │
├──────────────────────┼──────────────────────┼────────────────────────────────────────────────────────────────────────────────────┤
│ Jump                 │ jmp addr             │ Jump to the value stored in addr.                                                  │
├──────────────────────┼──────────────────────┼────────────────────────────────────────────────────────────────────────────────────┤
│ Jump If Equal        │ jz reg1,reg2,imm32   │ Jump imm32 bytes if the contents of reg1 are equal to the contents of reg2.        │
├──────────────────────┼──────────────────────┼────────────────────────────────────────────────────────────────────────────────────┤
│ Jump If Less Than    │ jl reg1,reg2,imm32   │ Jump imm32 bytes if the contents of reg1 are smaller than to the contents of reg2. │
├──────────────────────┼──────────────────────┼────────────────────────────────────────────────────────────────────────────────────┤
│ Jump If Greater Than │ jg reg1,reg2,imm32   │ Jump imm32 bytes if the contents of reg1 are larger than to the contents of reg2.  │
├──────────────────────┼──────────────────────┼────────────────────────────────────────────────────────────────────────────────────┤
│ Return               │ ret                  │ Transfer control to the address in the top of the stack.                           │
├──────────────────────┼──────────────────────┼────────────────────────────────────────────────────────────────────────────────────┤
│ Call                 │ call reg             │ Transfer control to the address contained in reg.                                  │
└──────────────────────┴──────────────────────┴────────────────────────────────────────────────────────────────────────────────────┘
```

Before starting to generate machine code for these instructions we need to
clearly define their operands.

The easiest operand kind to understand are the immediates `imm32` and `imm64`
which are just constant signed integer values. We represent them with the `i32`
and `i64` types in Rust.

Then we have registers or `reg`. The `x86_64` architecture has 16 general
purpose registers in 64-bit mode: `rax`, `rcx`, `rdx`, `rbx`, `rsp`, `rbp`,
`rsi`, `rdi` `r8`, ..., `r14` and `r15`, we will only use the first 8 for now.
Additionally we have the `rip` register which holds the instruction pointer.
There are other specific purpose registers that we will discuss if we need
them.

Finally, we have addresses or `addr` which represent memory locations. For now,
we will say that base addresses are stored in a register, we will extend this
later. The base address can be modified by adding an offset to the base
address, this offset can only be an `imm32` and not an `imm64` (this is a
limitation of the `x86` instruction set).

Now we are ready to encode those instructions as valid `x86_64` machine code.
These are the instructions that we will use taken from the Intel's manual:

```
┌───────────────────┬─────────────────┬───────┬───────────────────────────────────────────────────────┐
│ Opcode            │ Instruction     │ Op/En │ Description                                           │
├───────────────────┼─────────────────┼───────┼───────────────────────────────────────────────────────┤
│ REX.W + B8+ rd io │ MOV r64, imm64  │ OI    │ Move imm64 to r64.                                    │
├───────────────────┼─────────────────┼───────┼───────────────────────────────────────────────────────┤
│ REX.W + 8B /r     │ MOV r64,r/m64   │ RM    │ Move r/m64 to r64.                                    │
├───────────────────┼─────────────────┼───────┼───────────────────────────────────────────────────────┤
│ REX.W + 89 /r     │ MOV r/m64,r64   │ MR    │ Move r64 to m/r64.                                    │
├───────────────────┼─────────────────┼───────┼───────────────────────────────────────────────────────┤
│ FF /6             │ PUSH r/m64      │ M     │ Push r/m64.                                           │
├───────────────────┼─────────────────┼───────┼───────────────────────────────────────────────────────┤
│ 8F /0             │ POP r/m64       │ M     │ Pop top of stack into m64; increment stack pointer.   │
├───────────────────┼─────────────────┼───────┼───────────────────────────────────────────────────────┤
│ REX.W + 01 /r     │ ADD r/m64,r64   │ MR    │ Add r64 to r/m64.                                     │
├───────────────────┼─────────────────┼───────┼───────────────────────────────────────────────────────┤
│ REX.W + 81 /0 id  │ ADD r/m64,imm32 │ MI    │ Add imm32 sign-extended to 64-bits to r/m64.          │
├───────────────────┼─────────────────┼───────┼───────────────────────────────────────────────────────┤
│ REX.W + 05 id     │ ADD RAX,imm32   │ I     │ Add imm32 sign-extended to 64-bits to RAX.            │
├───────────────────┼─────────────────┼───────┼───────────────────────────────────────────────────────┤
│ FF /4             │ JMP r/m64       │ M     │ Jump near, absolute indirect.                         │
├───────────────────┼─────────────────┼───────┼───────────────────────────────────────────────────────┤
│ REX.W + 39 /r     │ CMP r/m64,r64   │ MR    │ Compare r64 with r/m64.                               │
├───────────────────┼─────────────────┼───────┼───────────────────────────────────────────────────────┤
│ 0F 84 cd          │ JE rel32        │ D     │ Jump near if equal.                                   │
├───────────────────┼─────────────────┼───────┼───────────────────────────────────────────────────────┤
│ 0F 8C cd          │ JL rel32        │ D     │ Jump near if less.                                    │
├───────────────────┼─────────────────┼───────┼───────────────────────────────────────────────────────┤
│ 0F 8F cd          │ JG rel32        │ D     │ Jump near if greater                                  │
├───────────────────┼─────────────────┼───────┼───────────────────────────────────────────────────────┤
│ C3                │ RET             │ ZO    │ Near return to calling procedure.                     │
├───────────────────┼─────────────────┼───────┼───────────────────────────────────────────────────────┤
│ FF /2             │ CALL r/m64      │ M     │ Call near, absolute indirect, address given in r/m64. │
└───────────────────┴─────────────────┴───────┴───────────────────────────────────────────────────────┘
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

To check that we generated the right machine code we can generate another
function in our object file which contains the instruction `loadi -1,reg` for
every `reg` and then dissasemble it with `objdump`:
```objdump
Disassembly of section .text:

0000000000000020 <loadi_test>:
  20:   48 b8 ef be ad de 00 00 00 00   movabs rax,0xdeadbeef
  2a:   48 b9 ef be ad de 00 00 00 00   movabs rcx,0xdeadbeef
  34:   48 ba ef be ad de 00 00 00 00   movabs rdx,0xdeadbeef
  3e:   48 bb ef be ad de 00 00 00 00   movabs rbx,0xdeadbeef
  48:   48 bc ef be ad de 00 00 00 00   movabs rsp,0xdeadbeef
  52:   48 bd ef be ad de 00 00 00 00   movabs rbp,0xdeadbeef
  5c:   48 be ef be ad de 00 00 00 00   movabs rsi,0xdeadbeef
  66:   48 bf ef be ad de 00 00 00 00   movabs rdi,0xdeadbeef
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

With this information we can build the `rsp` register setting the `index` to
zero, the `base` to the `rsp` register and the `scale` to 1 (we could use any
scale because it is multiplied by the index which is zero).

With this particular case solved. We are done encoding the load address
instruction and we are ready to test it by emitting code that calls this
instruction using all the possible register pairs and dissassembling it:
```objdump
Disassembly of section .text:

0000000000000070 <loada_test>:
  70:   48 8b 80 ef be 00 00    mov    rax,QWORD PTR [rax+0xbeef]
  77:   48 8b 81 ef be 00 00    mov    rax,QWORD PTR [rcx+0xbeef]
  7e:   48 8b 82 ef be 00 00    mov    rax,QWORD PTR [rdx+0xbeef]
  85:   48 8b 83 ef be 00 00    mov    rax,QWORD PTR [rbx+0xbeef]
  8c:   48 8b 84 24 ef be 00 00         mov    rax,QWORD PTR [rsp+0xbeef]
  94:   48 8b 85 ef be 00 00    mov    rax,QWORD PTR [rbp+0xbeef]
  9b:   48 8b 86 ef be 00 00    mov    rax,QWORD PTR [rsi+0xbeef]
  a2:   48 8b 87 ef be 00 00    mov    rax,QWORD PTR [rdi+0xbeef]
  a9:   48 8b 88 ef be 00 00    mov    rcx,QWORD PTR [rax+0xbeef]
  b0:   48 8b 89 ef be 00 00    mov    rcx,QWORD PTR [rcx+0xbeef]
  b7:   48 8b 8a ef be 00 00    mov    rcx,QWORD PTR [rdx+0xbeef]
  be:   48 8b 8b ef be 00 00    mov    rcx,QWORD PTR [rbx+0xbeef]
  c5:   48 8b 8c 24 ef be 00 00         mov    rcx,QWORD PTR [rsp+0xbeef]
  cd:   48 8b 8d ef be 00 00    mov    rcx,QWORD PTR [rbp+0xbeef]
  d4:   48 8b 8e ef be 00 00    mov    rcx,QWORD PTR [rsi+0xbeef]
  db:   48 8b 8f ef be 00 00    mov    rcx,QWORD PTR [rdi+0xbeef]
  e2:   48 8b 90 ef be 00 00    mov    rdx,QWORD PTR [rax+0xbeef]
  e9:   48 8b 91 ef be 00 00    mov    rdx,QWORD PTR [rcx+0xbeef]
  f0:   48 8b 92 ef be 00 00    mov    rdx,QWORD PTR [rdx+0xbeef]
  f7:   48 8b 93 ef be 00 00    mov    rdx,QWORD PTR [rbx+0xbeef]
  fe:   48 8b 94 24 ef be 00 00         mov    rdx,QWORD PTR [rsp+0xbeef]
 106:   48 8b 95 ef be 00 00    mov    rdx,QWORD PTR [rbp+0xbeef]
 10d:   48 8b 96 ef be 00 00    mov    rdx,QWORD PTR [rsi+0xbeef]
 114:   48 8b 97 ef be 00 00    mov    rdx,QWORD PTR [rdi+0xbeef]
 11b:   48 8b 98 ef be 00 00    mov    rbx,QWORD PTR [rax+0xbeef]
 122:   48 8b 99 ef be 00 00    mov    rbx,QWORD PTR [rcx+0xbeef]
 129:   48 8b 9a ef be 00 00    mov    rbx,QWORD PTR [rdx+0xbeef]
 130:   48 8b 9b ef be 00 00    mov    rbx,QWORD PTR [rbx+0xbeef]
 137:   48 8b 9c 24 ef be 00 00         mov    rbx,QWORD PTR [rsp+0xbeef]
 13f:   48 8b 9d ef be 00 00    mov    rbx,QWORD PTR [rbp+0xbeef]
 146:   48 8b 9e ef be 00 00    mov    rbx,QWORD PTR [rsi+0xbeef]
 14d:   48 8b 9f ef be 00 00    mov    rbx,QWORD PTR [rdi+0xbeef]
 154:   48 8b a0 ef be 00 00    mov    rsp,QWORD PTR [rax+0xbeef]
 15b:   48 8b a1 ef be 00 00    mov    rsp,QWORD PTR [rcx+0xbeef]
 162:   48 8b a2 ef be 00 00    mov    rsp,QWORD PTR [rdx+0xbeef]
 169:   48 8b a3 ef be 00 00    mov    rsp,QWORD PTR [rbx+0xbeef]
 170:   48 8b a4 24 ef be 00 00         mov    rsp,QWORD PTR [rsp+0xbeef]
 178:   48 8b a5 ef be 00 00    mov    rsp,QWORD PTR [rbp+0xbeef]
 17f:   48 8b a6 ef be 00 00    mov    rsp,QWORD PTR [rsi+0xbeef]
 186:   48 8b a7 ef be 00 00    mov    rsp,QWORD PTR [rdi+0xbeef]
 18d:   48 8b a8 ef be 00 00    mov    rbp,QWORD PTR [rax+0xbeef]
 194:   48 8b a9 ef be 00 00    mov    rbp,QWORD PTR [rcx+0xbeef]
 19b:   48 8b aa ef be 00 00    mov    rbp,QWORD PTR [rdx+0xbeef]
 1a2:   48 8b ab ef be 00 00    mov    rbp,QWORD PTR [rbx+0xbeef]
 1a9:   48 8b ac 24 ef be 00 00         mov    rbp,QWORD PTR [rsp+0xbeef]
 1b1:   48 8b ad ef be 00 00    mov    rbp,QWORD PTR [rbp+0xbeef]
 1b8:   48 8b ae ef be 00 00    mov    rbp,QWORD PTR [rsi+0xbeef]
 1bf:   48 8b af ef be 00 00    mov    rbp,QWORD PTR [rdi+0xbeef]
 1c6:   48 8b b0 ef be 00 00    mov    rsi,QWORD PTR [rax+0xbeef]
 1cd:   48 8b b1 ef be 00 00    mov    rsi,QWORD PTR [rcx+0xbeef]
 1d4:   48 8b b2 ef be 00 00    mov    rsi,QWORD PTR [rdx+0xbeef]
 1db:   48 8b b3 ef be 00 00    mov    rsi,QWORD PTR [rbx+0xbeef]
 1e2:   48 8b b4 24 ef be 00 00         mov    rsi,QWORD PTR [rsp+0xbeef]
 1ea:   48 8b b5 ef be 00 00    mov    rsi,QWORD PTR [rbp+0xbeef]
 1f1:   48 8b b6 ef be 00 00    mov    rsi,QWORD PTR [rsi+0xbeef]
 1f8:   48 8b b7 ef be 00 00    mov    rsi,QWORD PTR [rdi+0xbeef]
 1ff:   48 8b b8 ef be 00 00    mov    rdi,QWORD PTR [rax+0xbeef]
 206:   48 8b b9 ef be 00 00    mov    rdi,QWORD PTR [rcx+0xbeef]
 20d:   48 8b ba ef be 00 00    mov    rdi,QWORD PTR [rdx+0xbeef]
 214:   48 8b bb ef be 00 00    mov    rdi,QWORD PTR [rbx+0xbeef]
 21b:   48 8b bc 24 ef be 00 00         mov    rdi,QWORD PTR [rsp+0xbeef]
 223:   48 8b bd ef be 00 00    mov    rdi,QWORD PTR [rbp+0xbeef]
 22a:   48 8b be ef be 00 00    mov    rdi,QWORD PTR [rsi+0xbeef]
 231:   48 8b bf ef be 00 00    mov    rdi,QWORD PTR [rdi+0xbeef]
```

We can move to the next instruction now.

#### Store

Store is the inverse of the load address instruction so it is reasonable to use
the instruction `MOV r/m64,r64`. Actually, this was the example instruction
that we used to understand how to emit machine code. So it should be pretty
straightforward.

We have to set the `REX.W` byte to enable 64-bit mode, the first operand is
encoded in the `r/m` field and that the second operand is encoded in the `reg`
field. This means that we can reuse most of the code that we did for the load
address instruction because the arguments are flipped.

We can test this instruction by doing the same we did for the load address
instruction:
```objdump
Disassembly of section .text:

0000000000000240 <store_test>:
 240:   48 89 80 ef be 00 00    mov    QWORD PTR [rax+0xbeef],rax
 247:   48 89 88 ef be 00 00    mov    QWORD PTR [rax+0xbeef],rcx
 24e:   48 89 90 ef be 00 00    mov    QWORD PTR [rax+0xbeef],rdx
 255:   48 89 98 ef be 00 00    mov    QWORD PTR [rax+0xbeef],rbx
 25c:   48 89 a0 ef be 00 00    mov    QWORD PTR [rax+0xbeef],rsp
 263:   48 89 a8 ef be 00 00    mov    QWORD PTR [rax+0xbeef],rbp
 26a:   48 89 b0 ef be 00 00    mov    QWORD PTR [rax+0xbeef],rsi
 271:   48 89 b8 ef be 00 00    mov    QWORD PTR [rax+0xbeef],rdi
 278:   48 89 81 ef be 00 00    mov    QWORD PTR [rcx+0xbeef],rax
 27f:   48 89 89 ef be 00 00    mov    QWORD PTR [rcx+0xbeef],rcx
 286:   48 89 91 ef be 00 00    mov    QWORD PTR [rcx+0xbeef],rdx
 28d:   48 89 99 ef be 00 00    mov    QWORD PTR [rcx+0xbeef],rbx
 294:   48 89 a1 ef be 00 00    mov    QWORD PTR [rcx+0xbeef],rsp
 29b:   48 89 a9 ef be 00 00    mov    QWORD PTR [rcx+0xbeef],rbp
 2a2:   48 89 b1 ef be 00 00    mov    QWORD PTR [rcx+0xbeef],rsi
 2a9:   48 89 b9 ef be 00 00    mov    QWORD PTR [rcx+0xbeef],rdi
 2b0:   48 89 82 ef be 00 00    mov    QWORD PTR [rdx+0xbeef],rax
 2b7:   48 89 8a ef be 00 00    mov    QWORD PTR [rdx+0xbeef],rcx
 2be:   48 89 92 ef be 00 00    mov    QWORD PTR [rdx+0xbeef],rdx
 2c5:   48 89 9a ef be 00 00    mov    QWORD PTR [rdx+0xbeef],rbx
 2cc:   48 89 a2 ef be 00 00    mov    QWORD PTR [rdx+0xbeef],rsp
 2d3:   48 89 aa ef be 00 00    mov    QWORD PTR [rdx+0xbeef],rbp
 2da:   48 89 b2 ef be 00 00    mov    QWORD PTR [rdx+0xbeef],rsi
 2e1:   48 89 ba ef be 00 00    mov    QWORD PTR [rdx+0xbeef],rdi
 2e8:   48 89 83 ef be 00 00    mov    QWORD PTR [rbx+0xbeef],rax
 2ef:   48 89 8b ef be 00 00    mov    QWORD PTR [rbx+0xbeef],rcx
 2f6:   48 89 93 ef be 00 00    mov    QWORD PTR [rbx+0xbeef],rdx
 2fd:   48 89 9b ef be 00 00    mov    QWORD PTR [rbx+0xbeef],rbx
 304:   48 89 a3 ef be 00 00    mov    QWORD PTR [rbx+0xbeef],rsp
 30b:   48 89 ab ef be 00 00    mov    QWORD PTR [rbx+0xbeef],rbp
 312:   48 89 b3 ef be 00 00    mov    QWORD PTR [rbx+0xbeef],rsi
 319:   48 89 bb ef be 00 00    mov    QWORD PTR [rbx+0xbeef],rdi
 320:   48 89 84 24 ef be 00 00         mov    QWORD PTR [rsp+0xbeef],rax
 328:   48 89 8c 24 ef be 00 00         mov    QWORD PTR [rsp+0xbeef],rcx
 330:   48 89 94 24 ef be 00 00         mov    QWORD PTR [rsp+0xbeef],rdx
 338:   48 89 9c 24 ef be 00 00         mov    QWORD PTR [rsp+0xbeef],rbx
 340:   48 89 a4 24 ef be 00 00         mov    QWORD PTR [rsp+0xbeef],rsp
 348:   48 89 ac 24 ef be 00 00         mov    QWORD PTR [rsp+0xbeef],rbp
 350:   48 89 b4 24 ef be 00 00         mov    QWORD PTR [rsp+0xbeef],rsi
 358:   48 89 bc 24 ef be 00 00         mov    QWORD PTR [rsp+0xbeef],rdi
 360:   48 89 85 ef be 00 00    mov    QWORD PTR [rbp+0xbeef],rax
 367:   48 89 8d ef be 00 00    mov    QWORD PTR [rbp+0xbeef],rcx
 36e:   48 89 95 ef be 00 00    mov    QWORD PTR [rbp+0xbeef],rdx
 375:   48 89 9d ef be 00 00    mov    QWORD PTR [rbp+0xbeef],rbx
 37c:   48 89 a5 ef be 00 00    mov    QWORD PTR [rbp+0xbeef],rsp
 383:   48 89 ad ef be 00 00    mov    QWORD PTR [rbp+0xbeef],rbp
 38a:   48 89 b5 ef be 00 00    mov    QWORD PTR [rbp+0xbeef],rsi
 391:   48 89 bd ef be 00 00    mov    QWORD PTR [rbp+0xbeef],rdi
 398:   48 89 86 ef be 00 00    mov    QWORD PTR [rsi+0xbeef],rax
 39f:   48 89 8e ef be 00 00    mov    QWORD PTR [rsi+0xbeef],rcx
 3a6:   48 89 96 ef be 00 00    mov    QWORD PTR [rsi+0xbeef],rdx
 3ad:   48 89 9e ef be 00 00    mov    QWORD PTR [rsi+0xbeef],rbx
 3b4:   48 89 a6 ef be 00 00    mov    QWORD PTR [rsi+0xbeef],rsp
 3bb:   48 89 ae ef be 00 00    mov    QWORD PTR [rsi+0xbeef],rbp
 3c2:   48 89 b6 ef be 00 00    mov    QWORD PTR [rsi+0xbeef],rsi
 3c9:   48 89 be ef be 00 00    mov    QWORD PTR [rsi+0xbeef],rdi
 3d0:   48 89 87 ef be 00 00    mov    QWORD PTR [rdi+0xbeef],rax
 3d7:   48 89 8f ef be 00 00    mov    QWORD PTR [rdi+0xbeef],rcx
 3de:   48 89 97 ef be 00 00    mov    QWORD PTR [rdi+0xbeef],rdx
 3e5:   48 89 9f ef be 00 00    mov    QWORD PTR [rdi+0xbeef],rbx
 3ec:   48 89 a7 ef be 00 00    mov    QWORD PTR [rdi+0xbeef],rsp
 3f3:   48 89 af ef be 00 00    mov    QWORD PTR [rdi+0xbeef],rbp
 3fa:   48 89 b7 ef be 00 00    mov    QWORD PTR [rdi+0xbeef],rsi
 401:   48 89 bf ef be 00 00    mov    QWORD PTR [rdi+0xbeef],rdi
```

#### Push

The push instruction is the first instruction that we can basically copy from
the `x86_64` manual. We will use the `PUSH r/m64` instruction which encodes its
operand in the `r/m` field. The `/6` in the opcode means that the `reg` field
must be set to `0x6` to encode this instruction.

We test this in the same as way we did with the `loadi` instruction:
```objdump
Disassembly of section .text:

0000000000000410 <push_test>:
 410:   ff f0                   push   rax
 412:   ff f1                   push   rcx
 414:   ff f2                   push   rdx
 416:   ff f3                   push   rbx
 418:   ff f4                   push   rsp
 41a:   ff f5                   push   rbp
 41c:   ff f6                   push   rsi
 41e:   ff f7                   push   rdi
```

#### Pop

We can also copy the pop instruction from the manual. The actual instruction is
`POP r/m64`, the operand is encoded in the `r/m` field. Analogous to the push
instruction, the `/0` in the opcode means that the `reg` field is set to zero.

We test this instruction in the same way as before:
```objdump
Disassembly of section .text:

0000000000000420 <pop_test>:
 420:   8f c0                   pop    rax
 422:   8f c1                   pop    rcx
 424:   8f c2                   pop    rdx
 426:   8f c3                   pop    rbx
 428:   8f c4                   pop    rsp
 42a:   8f c5                   pop    rbp
 42c:   8f c6                   pop    rsi
 42e:   8f c7                   pop    rdi
```

#### Add

The instruction we will use for add is `ADD r/m64, r64` which encodes the first
operand in the `r/m` field and the second one in the `reg` field. We have to be
careful with the syntax order but other than that, emitting this instruction
should be straightforward.

We test this instruction in the same way as we did with the load address
instruction:

```objdump
Disassembly of section .text:

0000000000000430 <add_test>:
 430:   48 01 c0                add    rax,rax
 433:   48 01 c1                add    rcx,rax
 436:   48 01 c2                add    rdx,rax
 439:   48 01 c3                add    rbx,rax
 43c:   48 01 c4                add    rsp,rax
 43f:   48 01 c5                add    rbp,rax
 442:   48 01 c6                add    rsi,rax
 445:   48 01 c7                add    rdi,rax
 448:   48 01 c8                add    rax,rcx
 44b:   48 01 c9                add    rcx,rcx
 44e:   48 01 ca                add    rdx,rcx
 451:   48 01 cb                add    rbx,rcx
 454:   48 01 cc                add    rsp,rcx
 457:   48 01 cd                add    rbp,rcx
 45a:   48 01 ce                add    rsi,rcx
 45d:   48 01 cf                add    rdi,rcx
 460:   48 01 d0                add    rax,rdx
 463:   48 01 d1                add    rcx,rdx
 466:   48 01 d2                add    rdx,rdx
 469:   48 01 d3                add    rbx,rdx
 46c:   48 01 d4                add    rsp,rdx
 46f:   48 01 d5                add    rbp,rdx
 472:   48 01 d6                add    rsi,rdx
 475:   48 01 d7                add    rdi,rdx
 478:   48 01 d8                add    rax,rbx
 47b:   48 01 d9                add    rcx,rbx
 47e:   48 01 da                add    rdx,rbx
 481:   48 01 db                add    rbx,rbx
 484:   48 01 dc                add    rsp,rbx
 487:   48 01 dd                add    rbp,rbx
 48a:   48 01 de                add    rsi,rbx
 48d:   48 01 df                add    rdi,rbx
 490:   48 01 e0                add    rax,rsp
 493:   48 01 e1                add    rcx,rsp
 496:   48 01 e2                add    rdx,rsp
 499:   48 01 e3                add    rbx,rsp
 49c:   48 01 e4                add    rsp,rsp
 49f:   48 01 e5                add    rbp,rsp
 4a2:   48 01 e6                add    rsi,rsp
 4a5:   48 01 e7                add    rdi,rsp
 4a8:   48 01 e8                add    rax,rbp
 4ab:   48 01 e9                add    rcx,rbp
 4ae:   48 01 ea                add    rdx,rbp
 4b1:   48 01 eb                add    rbx,rbp
 4b4:   48 01 ec                add    rsp,rbp
 4b7:   48 01 ed                add    rbp,rbp
 4ba:   48 01 ee                add    rsi,rbp
 4bd:   48 01 ef                add    rdi,rbp
 4c0:   48 01 f0                add    rax,rsi
 4c3:   48 01 f1                add    rcx,rsi
 4c6:   48 01 f2                add    rdx,rsi
 4c9:   48 01 f3                add    rbx,rsi
 4cc:   48 01 f4                add    rsp,rsi
 4cf:   48 01 f5                add    rbp,rsi
 4d2:   48 01 f6                add    rsi,rsi
 4d5:   48 01 f7                add    rdi,rsi
 4d8:   48 01 f8                add    rax,rdi
 4db:   48 01 f9                add    rcx,rdi
 4de:   48 01 fa                add    rdx,rdi
 ```

#### Add Immediate

We will use the `ADD r/m64,imm32` instruction which adds the `imm32` value to
the contents of `r/m64`. This instruction is encoded by setting the `REX.W`
byte and setting the `reg` field to `0x0`. Additionally, the first operand is
encoded in the `r/m` field and the second operand is encoded as bytes after the
`mod r/m` byte.

However, our `addi` instruction has a particular case that can be written using
fewer bytes thanks to the `ADD RAX,imm32` instruction (yes, `x86_64` has a
specific "add to `rax`" instruction).

We test this instruction in the same way as we did with the load immediate
instruction:

```objdump
Disassembly of section .text:

0000000000000030 <addi_test>:
  30:   48 05 ef be ad de       add    rax,0xffffffffdeadbeef
  36:   48 81 c1 ef be ad de    add    rcx,0xffffffffdeadbeef
  3d:   48 81 c2 ef be ad de    add    rdx,0xffffffffdeadbeef
  44:   48 81 c3 ef be ad de    add    rbx,0xffffffffdeadbeef
  4b:   48 81 c4 ef be ad de    add    rsp,0xffffffffdeadbeef
  52:   48 81 c5 ef be ad de    add    rbp,0xffffffffdeadbeef
  59:   48 81 c6 ef be ad de    add    rsi,0xffffffffdeadbeef
  60:   48 81 c7 ef be ad de    add    rdi,0xffffffffdeadbeef
```

#### Jump

We will use the `JMP r/m64` instruction which from the `/4` in the opcode, we
know that we must set the `rem` field to `0x4`. The operand is encoded in the
`r/m` field.

We test this instruction in the same way as the other single operand instructions:
```objdump
Disassembly of section .text:

00000000000004f0 <jmp_test>:
 4f0:   ff e0                   jmp    rax
 4f2:   ff e1                   jmp    rcx
 4f4:   ff e2                   jmp    rdx
 4f6:   ff e3                   jmp    rbx
 4f8:   ff e4                   jmp    rsp
 4fa:   ff e5                   jmp    rbp
 4fc:   ff e6                   jmp    rsi
 4fe:   ff e7                   jmp    rdi
```

#### Conditional Jumps

We will start with the Jump If Zero instruction. This instruction performs a
conditional jump if the value contained in the second operand is zero. The
final position of the jump is in the first operand but this is relative to the
current location (in other words, the position of the instruction pointer after
reading this instruction).

This is an interesting instruction because we cannot encode it as a single
`x86_64` instruction. Conditional jumps in `x86` are done by first comparing
two operands. The result of this comparison is stored in the special `RFLAGS`
register. The actual conditional jump instruction uses the `RFLAGS` register to
decide to jump or not. This `RFLAGS` register is a sequence of status flags
indicating the result of the comparison.

So the first thing we need to do emit a compare instruction between our operand
and zero. We will use the `CMP r/m64,r64` which encodes the first operand in
the `r/m` field and the second operand in the `reg` field. This instruction
substracts the second operand from the first and sets the status flags in the
`RFLAGS` register based on the result of the substraction.

We can use this instruction and then use the `JE rel32` or jump if equal
instruction which encodes the operand by appending it after the `0x84` byte.

In other words, we will emit the following code to encode our `jz imm32,reg`:

```nasm
cmp  reg2,reg1 ; compare `reg1` to `reg2`.
je   imm32   ; if `reg1 == reg2`, jump to `imm32`.
```

We test this instruction in the same way as the others:
```objdump
Disassembly of section .text:

0000000000000500 <jz_test>:
 500:   48 83 f8 00             cmp    rax,0x0
 504:   0f 84 ef be 00 00       je     c3f9 <jz_test+0xbef9>
 50a:   48 83 f9 00             cmp    rcx,0x0
 50e:   0f 84 ef be 00 00       je     c403 <jz_test+0xbf03>
 514:   48 83 fa 00             cmp    rdx,0x0
 518:   0f 84 ef be 00 00       je     c40d <jz_test+0xbf0d>
 51e:   48 83 fb 00             cmp    rbx,0x0
 522:   0f 84 ef be 00 00       je     c417 <jz_test+0xbf17>
 528:   48 83 fc 00             cmp    rsp,0x0
 52c:   0f 84 ef be 00 00       je     c421 <jz_test+0xbf21>
 532:   48 83 fd 00             cmp    rbp,0x0
 536:   0f 84 ef be 00 00       je     c42b <jz_test+0xbf2b>
 53c:   48 83 fe 00             cmp    rsi,0x0
 540:   0f 84 ef be 00 00       je     c435 <jz_test+0xbf35>
 546:   48 83 ff 00             cmp    rdi,0x0
 54a:   0f 84 ef be 00 00       je     c43f <jz_test+0xbf3f>
```

from the operand of the disassembled `je` instructions it is not clear that we
are emitting the right bytes because `objdump` automatically tries to compute
the absolute position of the jump. But we can check that every value is correct
by doing a bit of arithmetic.

The first instruction we emited is `jz rax,0xbeef`. For this instruction we
have that the operand of `je` is `<jz_test+0xbef9>`, this reads as "the
position obtained from adding `0xbef9` to the start of `jz_test`". We know that
`jz_test` starts at `0x500` and that the current position is `0x50a` (the
position immediately after the `je` instruction). In other words the, current
position is `<jz_test+0x0a>`. Which means that the relative offset between the
two positions is `0xbef9 - 0x0a` which is exactly `0xbeef`.

The other two instructions are encoded in a similar way but using the `JL
rel32` and `JG rel32` instructions instead of `JE rel32`.

#### Return

After all this madness we go back to a simple instruction. We will use the near
return instruction `RET` which takes no operands.

Testing it is really simple:
```objdump
Disassembly of section .text:

0000000000000550 <ret_test>:
 550:   c3                      ret
```

#### Call

For this last instruction we will use the near call instruction `CALL r/m64`
which encodes its operand in the `rm` field.

We test it in the same way as the other instructions:
```objdump
Disassembly of section .text:

0000000000000560 <call_test>:
 560:   ff d0                   call   rax
 562:   ff d1                   call   rcx
 564:   ff d2                   call   rdx
 566:   ff d3                   call   rbx
 568:   ff d4                   call   rsp
 56a:   ff d5                   call   rbp
 56c:   ff d6                   call   rsi
 56e:   ff d7                   call   rdi
```

And we are done with or tiny assembler!

### Testing our assembler

We will need to modify our assembler in the future to add new instructions or
to extend the instructions we already have. To be sure that we won't break the
assembler at least for the instructions we already have we will add some tests.

These tests will compare our assembled instructions to actual `x86_64`
instructions assembled by the Netwide Assembler or `nasm`, an `x86` assembler.

First we need to write `x86` assembly files and assemble them. To assemble the
`code.asm` file, we run:
```bash
$ nasm code.asm -o code.out
```

This generates a `code.out` binary file with the assembled instructions. We can
write a `build.rs` script to assemble these `*.asm` files everytime we change
them so we don't have to invoke `nasm` manually every single time we change
something.

After writing the tests for each instruction, I found some differences for
three of them: `push`, `pop` and `jz`.

The `push` and `pop` instructions are assembled by `nasm` using less bytes.
Using less bytes means we will have smaller binaries so it is worth to take a
look.

We used the `PUSH r/m64` and `POP r/m64` instructions but after checking the
`nasm` output and reading the `x86` manuals, we also have the `PUSH r64` and
`POP r64` instructions which only take registers as operands and thus have a
shorter and simpler encoding. Their opcodes are `50 +rd` and `58 +rd`
respectively, meaning that we can encode both instructions in a single byte by
taking `0x50` or `0x58` and adding to them the `rd` encoding of the register
operand.

The `jz` case is bit more interesting. The differences are in the emitted code
for the `je` instruction, specifically in the operand bytes. It seems that the
manuals say that the operand is relative to the position of the instruction
pointer but is the assembler responsability to compute the absolute position
and then use the absolute position as the operand.

I reached this conclusion because the difference between the outputs is always
on the least significant byte of the operand and the differences for each
instruction are 10, 20, 30, 40 and so on. The total length of the emitted code
is exactly 10 bytes: 4 bytes for the `cmp` instruction and `6` bytes for the
`je` instruction. Meaning that the difference between the operand bytes in the
two outputs is exactly the position of the instruction pointer.

Fortunately for us, the position of the instruction pointer is exactly the
length of the buffer we're using to emit the code, so we can easily add it to
the operand.

### Emitting functions

Now that we can emit valid `x86_64` machine code we should be able to write our
own functions inside it. To be able to do this we need to understand how
function calls and returns work. In other words, we need to understand the
calling convention. At least on Linux for `x86_64` machines, the calling
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
loadi 0x0,rax
add   rdi,rax
add   rdi,rax
ret
```

We first set the `rax` register to zero to be on the safe side and then add
`value` twice to `rax`.

If we link `main.c` against our object file and run it, we should get the same
output as with the `lib.c` library.
