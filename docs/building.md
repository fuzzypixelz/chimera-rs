# Build System

## build.rs

The cargo build script is responsable for the following:

- Building the LLVM project with MLIR enabled through `cmake`.
- Generating bindings to the `MLIR` C API through bindgen and linking _dynamically_ to the MLIR-C shared library.
- Generating the `LALRPOP` parser.

## x.py

Since the MLIR C API doesn't allow (as of now) for translating the LLVM dialect to actual LLVM IR,
we invoke the `mlir-translate` tool in `/llvm-project/build/bin` to produce the desired output. This
is then provided to `clang` which chooses the appropriate target triple and produces the final executable.
