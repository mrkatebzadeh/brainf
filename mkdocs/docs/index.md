# BrainF Compiler and Interpreter
Welcome to BrainF Compiler and Interpreter documentation.
For more infromation about me please visit my homepage [mrkatebzadeh.xyz](http://mrkatebzadeh.xyz).

## What is BrainF
[BrainF](https://en.wikipedia.org/wiki/Brainfuck) is the ungodly creation of Urban Müller. It is proven to be **`turing-complete`** and very simple as go lang. BrainF operates on an array of memory cells, also referred to as the tape, each initially set to zero. There is a pointer, initially pointing to the first memory cell. The commands are:

|Command | Description| Equivalent|
|---|---|---|
|>|Move the pointer to the right| ++ptr|
|<|Move the pointer to the left| --ptr|
|+|Increment the memory cell under the pointer| ++*ptr|
|-|Decrement the memory cell under the pointer| --*ptr|
|.|Output the character signified by the cell at the pointer| putchar(*ptr)|
|,|Input a character and store it in the cell at the pointer| *ptr = getchar()|
|[|Jump past the matching ] if the cell under the pointer is 0| while (*ptr) {|
|]|Jump back to the matching [ if the cell under the pointer is nonzero| }|

This program prints out the words *Hello World!*:
```
+++++ +++             Set Cell #0 to 8
[
  >++++               Add 4 to Cell #1; this will always set Cell #1 to 4
  [                   as the cell will be cleared by the loop
      >++             Add 4*2 to Cell #2
      >+++            Add 4*3 to Cell #3
      >+++            Add 4*3 to Cell #4
      >+              Add 4 to Cell #5
      <<<<-           Decrement the loop counter in Cell #1
  ]                   Loop till Cell #1 is zero
  >+                  Add 1 to Cell #2
  >+                  Add 1 to Cell #3
  >-                  Subtract 1 from Cell #4
  >>+                 Add 1 to Cell #6
  [<]                 Move back to the first zero cell you find; this will
                      be Cell #1 which was cleared by the previous loop
  <-                  Decrement the loop Counter in Cell #0
]                     Loop till Cell #0 is zero

The result of this is:
Cell No :   0   1   2   3   4   5   6
Contents:   0   0  72 104  88  32   8
Pointer :   ^

>>.                     Cell #2 has value 72 which is 'H'
>---.                   Subtract 3 from Cell #3 to get 101 which is 'e'
+++++ ++..+++.          Likewise for 'llo' from Cell #3
>>.                     Cell #5 is 32 for the space
<-.                     Subtract 1 from Cell #4 for 87 to give a 'W'
<.                      Cell #3 was set to 'o' from the end of 'Hello'
+++.----- -.----- ---.  Cell #3 for 'rl' and 'd'
>>+.                    Add 1 to Cell #5 gives us an exclamation point
>++.                    And finally a newline from Cell #6
```
We have a simple and fast interpreter and a complete compiler for BrainF.
The compiling phase is structured as follows:

```
BF source -> BF IR -> LLVM IR -> x86_32 Binary
```

## Getting started
Getting started is super easy. You can clone the repository:

```sh
$ git clone https://github.com/mrkatebzadeh/brainf_compiler.git
```

## Usage

You will need LLVM and Rust installed to compile brainf_compiler.

```sh
$ cargo build --release
```
Debug builds work, but large BF programs will take a long time
in speculative execution if brainf_compiler is compiled without optimizations. You
can disable this by passing `--opt=0` or `--opt=1` when running brainf_compiler.

Compiling-mode:

```sh
$ target/release/brainf_compiler -c samples/hello_world.bf
$ ./hello_world
Hello World!
```

Interpreting-mode:

```sh
$ target/release/brainf_compiler -i samples/hello_world.bf
Hello World!
```

By default, brainf_compiler compiles programs to executables that run on the
current machine. You can explicitly specify architecture using LLVM
target triples:

```sh
$ target/release/brainf_compiler -c samples/hello_world.bf --target=x86_64-pc-linux-gnu
```

### LLVM Version

LLVM 3.8+ is recommended, as there are known bugs with 3.7. Either
download a prebuilt LLVM, or build it as follows:

```sh
$ wget http://llvm.org/pre-releases/3.8.0/rc1/llvm-3.8.0rc1.src.tar.xz
$ tar -xf llvm-3.8.0rc1.src.tar.xz

$ mkdir -p ~/tmp/llvm_3_8_build
$ cd ~/tmp/llvm_3_8_build

$ cmake -G Ninja /path/to/untarred/llvm
$ ninja
```

brainf_compiler depends on llvm-sys, which compiles against whichever
`llvm-config` it finds.

```sh
$ export PATH=~/tmp/llvm_3_8_build:$PATH
$ cargo build --release
```

### Portability

brainf_compiler considers cells to be single bytes, and arithmetic wraps
around. As a result, `-` sets cell #0 to 255.

brainf_compiler provides 100,000 cells. Accessing cells outside of this range is
explicitly undefined, and will probably segfault your program. brainf_compiler
will generate a warning if it can statically prove out-of-range cell
access.

brainf_compiler requires brackets to be balanced, so `+[]]` is rejected, unlike
some BF interpreters.

Finally, brainf_compiler assumes input files are valid UTF-8.

## Diagnostics

brainf_compiler can report syntax errors and warnings with relevant line numbers
and highlighting.

Note that some warning are produced during optimization, so disabling
optimizations will reduce warnings.

## Optimizations

### Peephole optimizations

brainf_compiler provides a range of peephole optimizations. We use quickcheck to
ensure our optimizations are in the optimal order (by verifying that
our optimizer is idempotent).

#### Combining Instructions

We combine successive increments/decrements:

```
   Compile            Combine
+++  =>   Increment 1   =>   Increment 3
          Increment 1
          Increment 1
```

If increments/decrements cancel out, we remove them entirely.

```
   Compile             Combine
+-   =>   Increment  1    =>   # nothing!
          Increment -1
```

We combine pointer increments:

```
   Compile            Combine
+++  =>   PointerIncrement 1   =>   PointerIncrement 2
          PointerIncrement 1
```

We do the same thing for successive sets:

```
      Combine
Set 1   =>   Set 2
Set 2

```

We combine sets and increments too:

```
  Compile            Known zero       Combine
+   =>   Increment 1   =>   Set 0       =>   Set 1
                            Increment 1

```

We remove increments when there's a set immediately after:

```
            Combine
Increment 1   =>   Set 2
Set 2

```

We remove both increments and sets if there's a read immediately
after:

```
            Combine
Increment 1   =>   Read
Read

```

We track the current cell position in straight-line code. If we can
determine the last instruction to modify the current cell, it doesn't
need to be immediately previous. For example, `+>-<,`:

```
                   Combine
Increment 1          =>   PointerIncrement 1
PointerIncrement 1        Increment -1
Increment -1              PointerIncrement -1
PointerIncrement -1       Read
Read

```

#### Loop Simplification

`[-]` is a common BF idiom for zeroing cells. We replace that with
`Set`, enabling further instruction combination.

```
   Compile              Simplify
[-]  =>   Loop             =>   Set 0
            Increment -1
```

#### Dead Code Elimination

We remove loops that we know are dead.

For example, loops at the beginning of a program:

```
    Compile                  Known zero               DCE
[>]   =>    Loop                 =>     Set 0          => Set 0
              DataIncrement 1           Loop
                                            DataIncrement
```


Loops following another loop (one BF technique for comments is
`[-][this, is+a comment.]`).

```
      Compile                 Annotate                 DCE
[>][>]   =>  Loop                =>   Loop              =>   Loop
               DataIncrement 1          DataIncrement 1        DataIncrement 1
             Loop                     Set 0                  Set 0
               DataIncrement 1        Loop
                                          DataIncrement 1
```

Loops where the cell has previously been set to zero:

```
        Compile               Simplify                 DCE
[-]>+<[]  =>   Loop              =>    Set 0            =>  Set 0
                 Increment -1          DataIncrement 1      DataIncrement 1
               DataIncrement 1         Increment 1          Increment 1
               Increment 1             DataIncrement -1     DataIncrement -1
               DataIncrement -1        Loop
               Loop
```

We remove redundant set commands after loops (often generated by loop
annotation as above).

```
       Remove redundant set
Loop           =>   Loop
  Increment -1        Increment -1
Set 0

```

We also remove dead code at the end of a program.

```
        Remove pure code
Write         =>           Write
Increment 1
```

Finally, we remove cell modifications that are immediately overwritten
by reads, e.g. `+,` is equivalent to `,`.

#### Reorder with offsets

Given a sequence of instructions without loops or I/O, we can safely
reorder them to have the same effect (we assume no out-of-bound cell
access).

This enables us to combine pointer operations:

```
    Compile                   Reorder
>+>   =>   PointerIncrement 1   =>    Increment 1 (offset 1)
           Increment 1                PointerIncrement 2
           PointerIncrement 1
```

We also ensure we modify cells in a consistent order, to aid cache
locality. For example, `>+<+>>+` writes to cell #1, then cell #0, then
cell #2. We reorder these instructions to obtain:

```
Increment 1 (offset 0)
Increment 1 (offset 1)
Increment 1 (offset 2)
PointerIncrement 2
```

#### Multiply-move loops

brainf_compiler can detect loops that perform multiplication and converts them to
multiply instructions. This works for simple cases like `[->++<]`
(multiply by two into the next cell) as well as more complex cases
like `[>-<->>+++<<]`.

### Cell Bounds Analysis

BF programs can use up to 100,000 cells, all of which must be
zero-initialised. However, most programs don't use the whole range.

brainf_compiler uses static analysis to work out how many cells a BF program may
use, so it doesn't need to allocate or zero-initialise more memory
than necessary.

```
>><< only uses three cells
```

```
[>><<] uses three cells at most
```

```
[>><<]>>> uses four cells at most
```

```
[>] may use any number of cells, so we must assume 100,000
```

### Speculative Execution

brainf_compiler executes as much as it can at compile time. For some programs
(such as hello_world.bf) this optimizes away the entire program to
just writing to stdout. brainf_compiler doesn't even need to allocate memory for
cells in this situation.

```
$ cargo run -- samples/hello_world.bf -c --dump-llvm
@known_outputs = constant [13 x i8] c"Hello World!\0A"

declare i32 @write(i32, i8*, i32)

define i32 @main() {
entry:
  %0 = call i32 @write(i32 0, i8* getelementptr inbounds ([13 x i8]* @known_outputs, i32 0, i32 0), i32 13)
  ret i32 0
}
```

#### Infinite Loops

brainf_compiler sets a maximum number of execution steps, avoiding infinite loops
hanging the compiler. As a result `+[]` will have `+` executed (so our
initial cell value is `1` and `[]` will be in the compiled output.

#### Runtime Values

If a program reads from stdin, speculation execution stops. As a
result, `>,` will have `>` executed (setting the initial cell pointer
to 1) and `,` will be in the compiled output.

#### Loop Execution

If loops can be entirely executed at compile time, they will be
removed from the resulting binary. Partially executed loops will be
included in the output, but runtime execution can begin at an
arbitrary position in the loop.

For example, consider `+[-]+[+,]`. We can execute `+[-]+`
entirely, but `[+,]` depends on runtime values. The
compiled output contains `[+,]`, but we start execution at the
`,` (continuing execution from where compile time execution had to
stop).
