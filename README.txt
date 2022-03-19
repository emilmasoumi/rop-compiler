A return-oriented programming (ROP) compiler that compiles a domain-specific
programming language to a return-oriented program. Such program is an exploit
payload based on stack buffer overflows on binary executables consisting of
gadgets that can be chained together. The compiler can generically be applied
to compile any instruction-oriented program such as jump-oriented programs
(JOP).

The language is specified in the syntax and semantics section.

------------
Dependencies:
radare2

------------
Building:
make -j$(nproc)

------------
Usage:
./ropc <binaries> <endianess> <architecture> <architecture-size> [options]
arguments:
    <binaries>...:
      Source code file and binary file(s).
    <-a --arch> <architecture>:
      Instruction set architecture.
      <possible values: arm, x86>
    <-b --archsize> <architecture-size>:
      Architecture bit-size. <possible values: 8, 16, 32, 64>
    <-e --endianess> <endianess>:
      Endianess of the binary file(s). <possible values: lsb, msb>
    {-s --select} <syntax>:
      Select the assembly syntax. <possible values: att, intel>
options:
    {-c --cpu} <cpu-type>:
      Select a specific CPU type.
      <possible values: v8, cortex, r6, v3, v2, v9>
    {-d --delete} <bytes>:
      Delete all occurrences of the given byte from the payload.
    {-H --hex}:
      Insert hexadecimal escape sequences for every byte in the payload.
    {-h --help}:
      Print this usage message.
    {-P --ppast}:
      Pretty-print the abstract syntax tree.
    {-p --ppgadgets}:
      Pretty-print the gadget chain.

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

A gadget evaluates to the memory address of its instruction offset. A gadget
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
// ./ropc src.rop main -a x86 -b 64 -s att -e lsb

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
