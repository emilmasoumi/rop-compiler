A return-oriented programming (ROP) compiler that compiles a domain-specific
programming language to a return-oriented program. Such program is an exploit
payload based on stack buffer overflows in binary executables consisting of
gadgets that can be chained together. The compiler is more specifically a
gadget chain compiler, as it can generically be applied to interchangeably
compile any instruction(s)-oriented program such as jump-oriented programs (JOP)
or both ROP and JOP in the same gadget chain.

The compiler supports the architectures: ARM/ARM64, MIPS32/64, SPARC32/64,
x86/x64, and the file formats: ELF, PE, Mach-O.

The language is specified in the syntax and semantics section.

------------
Building:

make

------------
Usage:

./ropc <src-code> <binary> <cpu-type> [options]
arguments:
    <src-code>:
      The source code file.
    <binary>:
      The binary executable file.
    <-c --cputype> <cpu-type>:
      The computer architecture/CPU type of the binary executable file.
      <possible values: arm, thumb, armv8, micro, mips3, mips32r6, mips32,
                        mips64, sparc32, sparcv9, x86-16, x86-32, x86-64>
options:
    {-b --bytewise}:
      Search for memory addresses byte-wise instead of mnemonic-wise.
    {-h --help}:
      Print this usage message.
    {-e --byteorder}:
      Adjust the byte order of the addresses in the gadgets to adapt to the
      endianness of the architecture.
    {-i --individually}:
      Display the gadget and the addresses in the gadget chain individually.
    {-l --list}:
      Display every address present in the binary for all gadgets.
    {-s --select} <syntax>:
      The assembly syntax for dis/assembling. <possible values: att, gas,
      intel, nasm>
    <-w --bitwidth> <bit-width>:
      Extend the addresses in the gadgets to the computer architecture bit
      width of the binary.
      <possible values: 16, 32, 64>

------------
Syntax and semantics:

A gadget contains one or more sequences of machine code instructions that the
compiler will search for in the binary. Gadgets can contain multiple subgadgets
meaning that the compiler will use the first subgadget it can identify as a
gadget in the binary:

{
  "...", "..."
};

The assembly must be defined inside two string literals "...", for example:
"mov rsi, rax;". The instructions specified inside of string literals must be
syntactically correct for the respective machine architecture and its
correctness is ignored by the compiler.

Variables are declared when they are defined: `id = exp`. They must be defined
once referenced and defined identifiers inside string literals must be
prepended with `@` to be referenced. References are replaced with their actual
value. The following code:

rax = "rsi";
{ "mov @rax, @rax; ret;" };

evaluates to:

{ "mov rsi, rsi; ret;" };

This allows inserting the address of a gadget into a proceeding gadget.

Variables are mutable and cannot be explicitly typed. The compiler must
implement type inference at compile-time, as identifiers are implicitly typed
by the compiler. The compiler uses static type checking to ensure type safety.

Arrays are data structures that contain a certain amount of elements and cannot
be nested. Identifiers that are referenced inside an array are replaced with
their value. Arrays that are referenced inside arrays are replaced with their
contents and gadgets are replaced with their determined addresses. Arrays can
be defined as:

arr = ["add", "sub", "mul", "div"];

Arrays are evaluated to a set of subgadgets when referenced inside gadgets.
When referencing the above definition of `arr` in a gadget:

{ "@arr rsp, rsp;" };

it evaluates to the following:

{
  "add rsp, rsp;", "sub rsp, rsp;", "mul rsp, rsp;", "div rsp, rsp;"
};

Array expressions cannot be used inside gadgets. Every possible permutation is
mutated when multiple arrays are encountered in a gadget. The following code:

arr1 = ["push", "pop"];
arr2 = ["add", "sub"];
{ "@arr1 rax; @arr2 rbx, rcx;" };

evaluates to:

{
  "push rax; add rbx, rcx;", "pop rax; add rbx, rcx;",
  "push rax; sub rbx, rcx;", "pop rax; sub rbx, rcx;"
};

A gadget evaluates to the memory address offset of its instruction(s). A gadget
is inserted into the payload when it is referenced at a global scope:

gadget1 = { "pop rsi; ret;" };
gadget2 = { "call 0x59a3bf;" };

// First memory address in the payload.
gadget1;
// Second memory address in the payload.
gagdet2;
// Third memory address in the payload.
{ "pop rax; ret;" };

------------
Example:

// Assume x64 architecture and an executable binary that was compiled with:
// `gcc main.c -o main`.
// A ROP/JOP program can then be compiled with:
// ./ropc src.rop main -c x86-64 -s intel

r    = "ret";
regs = ["rax", "rcx", "rdx", "rbx", "rsp", "rbp", "rsi", "rdi",
        "r8 ", "r9 ", "r10", "r11", "r12", "r13", "r14", "r15"];

// gadget 1
{
  "lea rax, [@regs + 8]; add @regs, @regs; @r;"
};

push_ebx = { "push ebx; ret;" };

// gadget 2
push_ebx;

// gadget 3
{
  "call 0xa9ef23;"
};

------------
Author:

Emil Masoumi
