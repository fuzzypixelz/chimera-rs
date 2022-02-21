#!/usr/bin/python

"""
Compile and execute a Chimera program.
"""

from subprocess import run, DEVNULL


def main():
    """
    Ready. Set. Go!
    """
    # Compile the compiler
    run(["cargo", "build"], stderr=DEVNULL, check=True)
    # Compile Chimera to MLIR IR
    with open("out.mlir", "w+", encoding="utf-8") as out_mlir:
        run(
            ["./target/debug/chimera"],
            stdout=out_mlir,
            env={"LD_LIBRARY_PATH": "llvm-project/build/lib/"},
            check=True
        )
        print("[X] compiled Chimera to MLIR")
    # Compiler MLIR to LLVM IR
    with open("out.ll", "w+", encoding="utf-8") as out_ll:
        run([
            "./llvm-project/build/bin/mlir-translate",
            "--mlir-to-llvmir",
            "out.mlir"
        ], stdout=out_ll, check=True)
        print("[X] compiled MLIR to LLVM IR")
    # Compile LLVM IR to a dynamically linked native executable.
    # Since we intentionally don't set the target triple in LLVM,
    # we expect clang to do the dirty job and set it appropriatly.
    run(["clang", "-Wno-override-module", "out.ll", "-o", "out"], check=True)
    print("[X] program returned:", run(["./out"], check=False).returncode)


if __name__ == "__main__":
    main()
