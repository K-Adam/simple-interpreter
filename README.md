# Simple Interpreter

This project implements a Lexer, Parser and Interpreter for a simple language. An example script can be found in `example.txt`

## Build

To build the project, simply run:

```bash
cargo build --release
```

The executable can be found in target/release

## Run

The interpreter cli accepts a single argument that points to a source file, like the provided `example.txt`

```bash
cd target/release
```

Windows:

```bash
.\simple-interpreter.exe ..\..\example.txt
```

Linux:

```bash
./simple-interpreter ../../example.txt
```
