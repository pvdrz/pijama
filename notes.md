
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

Just to be sure we got everything right we will add a second function to the
`lib.c` file and try to replicate the object file. Here's the new `.text`
section:
```
Disassembly of section .text:

0000000000000000 <start>:
   0:   55                      push   %rbp
   1:   48 89 e5                mov    %rsp,%rbp
   4:   b8 0a 00 00 00          mov    $0xa,%eax
   9:   5d                      pop    %rbp
   a:   c3                      ret
   b:   0f 1f 44 00 00          nopl   0x0(%rax,%rax,1)

0000000000000010 <duplicate>:
  10:   55                      push   %rbp
  11:   48 89 e5                mov    %rsp,%rbp
  14:   89 7d fc                mov    %edi,-0x4(%rbp)
  17:   8b 45 fc                mov    -0x4(%rbp),%eax
  1a:   c1 e0 01                shl    $0x1,%eax
  1d:   5d                      pop    %rbp
  1e:   c3                      ret
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

The `Mod R/M` and `SIB` parts encode an operand of the instruction, this
operand resides in memory. These two parts specify how to access this operand.
Some instructions don't require this.

Sometimes the `Mod R/M` bytes are followed by a `Displacement` which can use 1,
2 or 4 bytes.

Finally if an instruction has an immediate operand it is encoded in the
`Immediate` part which can use 1, 2 or 4 bytes.

As an example, let's take a look at the `mov` instruction inside the `start`
function at position `1`:
```bash
$ objdump -M intel -D lib_2.o
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
We added the `-M intel` flag to be sure we are using the Intel syntax and not
the AT&T one. This is a major source of headaches because each syntax puts
source and destination operands in a different order. The machine code is `48
89 e5` and the instruction is `mov rbp,rsp`.

Sadly, there are more than 30 different `mov` instructions in the Intel manual.
But based on the fact that both operands are 64-bit registers and that it has
this `89`. We can assume that the entry in the manual is this one:

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
by an `89` byte. Finally `/r` means that the `Mod R/M` part has a register
operand and a register or memory location operand.

The `Instruction` column is a bit clearer. Is saying that this instruction is
usually shown as `MOV`, followed by a 64-bit register or memory operand and
then a register operand. In our case, both operands are registers.

The `Op/En` or operand encoding column specifies how the operands are encoded.
`MR` means that the first operand is encoded in the `R/M` part and that the
second operand is encoded in the `Reg` part.

The `64-Bit Mode` column says if this instruction supports in 64-bit mode or not.

Now we can try to reconstruct the instruction:

The `REX` prefix is composed of three bits `W`, `R`, `B` and is written as a
single byte with the following binary format `0b0100WR0B`. From the `Opcode`
column we can infer that we only need to set the `W` bit. Meaning that the
first byte of the instruction should be `0b01001000` or `0x48`.

The second byte is easy as the `Opcode` column says it is `0x89`.

The third byte is a `Mod R/M` byte and it must encode the `rbp` and `rsp`
registers. We know that this `Mod R/M` is divided in three parts:
- The `Mod` part which uses the bits 7-6.
- The `Reg/Opcode` part which uses the bits 5-3.
- The `R/M` part which uses the `2-0` bytes.

We gave this byte intervals "backwards" because we are in little-endian.

The `x86` manual has a table explaining how operands are encoded in these three
part and from the `Op/En` column we know that the first operand must be encoded
in the `R/M` part and the second in the `Reg` part.

To encode the `bp` register as the first operand we set `Mod` to `11` and `R/M`
to `101`. To encode the `sp` register as the second operand we set `Reg/Opcode`
to `100`. Meaning that the whole byte is `0b11100101` or `0xe5`.

So the machine code should be `48 89 e5`. Just like what we have in our
disassembled object file.

As we can see, the x86 instruction set is very complex and providing every
single instruction in the set would be an almost impossible task. The good news
is that given that we are writing our own assembler we can implement a subset
of it.

### Writing a small instruction set

For now we will only support 64-bit operands. Smaller operands can be supported by
either casting them to 64-bit integers or by extending our instruction set
later.

We will start with the following instructions

```
┌────────────────┬────────────────┬──────────────────────────────────────────────────────────────────────────────────────┐
│      Name      │   Instruction  │     Description                                                                      │
├────────────────┼────────────────┼──────────────────────────────────────────────────────────────────────────────────────┤
│ Load Immediate │ loadi imm,reg  │ Load the imm value into reg.                                                         │
├────────────────┼────────────────┼──────────────────────────────────────────────────────────────────────────────────────┤
│ Load Address   │ loada addr,reg │ Load the contents of addr into reg.                                                  │
├────────────────┼────────────────┼──────────────────────────────────────────────────────────────────────────────────────┤
│ Store          │ store reg,addr │ Store the contents of reg into addr.                                                 │
├────────────────┼────────────────┼──────────────────────────────────────────────────────────────────────────────────────┤
│ Push           │ push reg       │ Push the contents of reg into the stack.                                             │
├────────────────┼────────────────┼──────────────────────────────────────────────────────────────────────────────────────┤
│ Pop            │ pop reg        │ Pop a value from the stack and put it in reg.                                        │
├────────────────┼────────────────┼──────────────────────────────────────────────────────────────────────────────────────┤
│ Add            │ add reg1,reg2  │ Add the contents of reg1 to the contents of reg2.                                    │
├────────────────┼────────────────┼──────────────────────────────────────────────────────────────────────────────────────┤
│ Add Immediate  │ addi imm,reg   │ Add the imm value to the contents of reg.                                            │
├────────────────┼────────────────┼──────────────────────────────────────────────────────────────────────────────────────┤
│ Jump           │ jmp addr       │ Jump to the value stored in addr.                                                    │
├────────────────┼────────────────┼──────────────────────────────────────────────────────────────────────────────────────┤
│ Jump LEZ       │ jlez addr,reg  │ Jump to the value stored in addr if the contents of reg are less or equal than zero. │
├────────────────┼────────────────┼──────────────────────────────────────────────────────────────────────────────────────┤
│ Return         │ ret            │ Transfer control to the address in the top of the stack.                             │
├────────────────┼────────────────┼──────────────────────────────────────────────────────────────────────────────────────┤
│ Call           │ call reg       │ Transfer control to the address contained in reg.                                    │
└────────────────┴────────────────┴──────────────────────────────────────────────────────────────────────────────────────┘
```

Before starting to generate machine code for these instructions we need to
clearly define their operands.

The easiest operand kind to understand are immediates or `imm` which are just
constant values. For now we can represent them with a `i64` type.

Then we have registers or `reg`. The `x86_64` architecture has 16 general
purpose registers in 64-bit mode: `rax`, `rcx`, `rdx`, `rbx`, `rsp`, `rbp`,
`rsi`, `rdi` `r8`, ..., `r14` and `r15`, we will only use the first 8 for now.
Additionally we have the `rip` register which holds the instruction pointer.
There are other specific purpose registers that we will discuss if we need
them.

Finally, we have addresses or `addr` which represent memory locations. For now,
we will say that addresses are composed of a base address stored in a register
and an offset stored in a 32-bit signed integer (we only use 32 bits because
offsets are not supposed to be large).

Now we are ready to encode those instructions as valid `x86_64` machine code.

#### Load Immediate

To load an immedate to a register we will use the `MOV r64,imm64` instruction
which has the opcode `REX.W + B8+ rd io`. The first operand is encoded by
adding an `rd` value representing the register to the `0xB8` opcode and setting the
`REX.B` bit to zero. The second operand is encoded as the literal value.

The `rd` value changes from 0 to 7 for the first 8 general purpose registers in
the order they are written above.

To check that we generated the right machine code we can generate another
function in our object file which contains the instruction `loadi -1,reg` for
every `reg` and then dissasemble it with `objdump`:

```
0000000000000020 <asm_test>:
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
