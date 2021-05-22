A return-oriented programming (ROP) compiler that compiles a domain-specific
programming language (DSL) to a return-oriented program. Such program is an
exploit payload based on stack buffer overflows on binary executables
consisting of gadgets that can be chained together. The compiler can
generically be applied to compile any instruction-oriented program such as
jump-oriented programs (JOP).

The DSL is specified in the syntax and semantics sections.

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
      <possible values: arm, mips, sparc, wasm, x86>
    <-b --archsize> <architecture-size>:
      Architecture bit-size. <possible values: 8, 16, 32, 64>
    <-e --endianess> <endianess>:
      Endianess of the binary file(s). <possible values: lsb, msb>
    {-s --select} <syntax>:
      Select the assembly syntax. <possible values: att, intel>
options:
    {-c --cpu} <cpu-type>:
      Select a specific CPU type.
      <possible values: v8, cortex, mips32, mips64, micro, r6, v3, v2, v9>
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
Syntax:
Extended Backus-Naur form (EBNF):

digit = "0" | "1" | "2" | "3" | "4" | "5" | "6" | "7" | "8" | "9" ;

letter = "A" | "B" | "C" | "D" | "E" | "F" | "G"
       | "H" | "I" | "J" | "K" | "L" | "M" | "N"
       | "O" | "P" | "Q" | "R" | "S" | "T" | "U"
       | "V" | "W" | "X" | "Y" | "Z" | "a" | "b"
       | "c" | "d" | "e" | "f" | "g" | "h" | "i"
       | "j" | "k" | "l" | "m" | "n" | "o" | "p"
       | "q" | "r" | "s" | "t" | "u" | "v" | "w"
       | "x" | "y" | "z" ;

identifier = letter, { ( "_" | digit | letter ) } ;

symbol = "(" | ")" | "{" | "}" | "[" | "]" | "<" | ">"
       | "'" | """ | "=" | "|" | "." | "," | ";" | "!"
       | "#" | "$" | "%" ;

character = digit | letter | symbol | "_" ;

assembly = """, character, """ ;

block = "{", assembly, { ",", assembly }, "}" ;

array = assembly, ",", assembly, { ",", assembly } ;

declaration = "let", identifier, ";" ;

assignment = identifier, "=", ( array | block | identifier ), ";" ;

call = ( block | identifier ), ";" ;

definition = "let", identifier, "=", ( array | block | identifier ), ";" ;

comment = "/", "/", ( *-"\n" ), "\n" ;

------------
Semantics:

A block contains one or more sequences of machine code instructions that the
compiler will search for in the binary. A block can be considered equivalent to
a gadget in ROP. Blocks can contain multiple inner-blocks meaning that the
compiler will use the first inner-block it can identify as a gadget in the
binary:

{
  "...", "..."
};

The assembly must be defined inside two string literals "...", for example:
"mov rsi, rax;". The sequence of machine instructions specified inside of
string literals must be syntactically correct for the respective machine
architecture and its correctness is ignored by the parser.

Variables are declared with a `let id;` expression and can be defined with
`let id = val`. They must be defined when referenced and undeclared identifiers
inside string literals are treated as assembly constructs when referenced.
Such construct must hence exist in the instruction set architecture of the
given Executable and Linkable Format (ELF)/Portable Executable (PE) binary.
Identifiers shadow the instruction sets mnemonics, so the following code:

let rax = "rsi";
{ "mov rax, rax; ret;" };

evaluates to:

{ "mov rsi, rsi; ret;" };

on the x86 architecture when assembled. This allows inserting the address of a
gadget into a proceeding gadget.

Variables are immutable and cannot be explicitly typed. The compiler must
implement type inference that infers types at compile time since identifiers
are implicitly typed by the compiler. The compiler uses static type checking
to ensure type safety.

Arrays are data structures that contain a certain amount of elements and cannot
be nested. Identifiers that are referenced inside an array are replaced with
their value. Arrays that are referenced inside arrays are replaced with their
contents and blocks are replaced with their determined addresses. Arrays can be
defined as:

let arr = "add", "sub", "mul", "div";

Arrays are evaluated to a set of inner-blocks when referenced inside blocks.
When referencing the above definition of `arr` in a block:

{ "arr rsp, rsp;" };

it evaluates to the following:

{
  "add rsp, rsp;", "sub rsp, rsp;", "mul rsp, rsp;", "div rsp, rsp;"
};

Arrays cannot implicitly be defined inside blocks, meaning that the array
expression assigned to the identifier cannot be used instead of referencing the
respective array. Every possible permutation is mutated when multiple arrays
are encountered in a block. The following code:

let arr1 = "push", "pop";
let arr2 = "add", "sub";
{ "arr1 rax; arr2 rbx, rcx;" };

evaluates to:

{
  "push rax; add rbx, rcx;", "pop rax; add rbx, rcx;",
  "push rax; sub rbx, rcx;", "pop rax; sub rbx, rcx;"
};

A block evaluates to the memory address where its instructions will be located
at during runtime of the program. A block is inserted into the payload when it
is referenced at a global scope:

let gadget1 = { "pop rsi; ret;" };
let gadget2 = { "call 0x59a3bf;" };

// First memory address in the payload.
gadget1;
// Second memory address in the payload.
gagdet2;
// Third memory address in the payload.
{ "pop rax; ret;" };

------------
Example:

// Assume x86 architecture and an executable binary that was compiled with:
// `gcc main.c -o main`.
// A ROP or JOP program can then be compiled with:
// ./ropc src.rop main -a x86 -b 64 -s att -e lsb

// Refer to the `examples` directory for full examples based on real programs.

let r    = "ret";
let regs = "rax", "rcx", "rdx", "rbx", "rsp", "rbp", "rsi", "rdi",
           "r8 ", "r9 ", "r10", "r11", "r12", "r13", "r14", "r15";

// gadget 1
{
  "lea rax, [regs + 8]; add regs, regs; r;"
};

let push_ebx = { "push ebx; ret;" };

// gadget 2
push_ebx;

// gadget 3
{
  "call 0xa9ef23;"
}

------------
Author:

Emil Masoumi
