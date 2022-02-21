use std::env;
use std::path::PathBuf;

fn main() {
    // Compile the LLVM Project with MLIR enabled as per:
    //     https://mlir.llvm.org/getting_started/
    println!("cargo:rerun-if-changed=llvm-project/mlir");
    cmake::Config::new("llvm-project/llvm")
        .out_dir("llvm-project")
        .generator("Ninja")
        .define("LLVM_ENABLE_PROJECTS", "mlir")
        .define("LLVM_TARGETS_TO_BUILD", "X86")
        .define("CMAKE_BUILD_TYPE", "Release")
        .define("LLVM_ENABLE_ASSERTIONS", "ON")
        .define("CMAKE_C_COMPILER", "clang")
        .define("CMAKE_CXX_COMPILER", "clang++")
        .define("LLVM_ENABLE_LLD", "ON")
        .define("MLIR_BUILD_MLIR_C_DYLIB", "ON")
        .target("check-mlir")
        .build();

    // "Linking statically is going to be the typical problem with trying
    // to use CMake libraries from outside of CMake: they are really not
    // designed to be used outside of the build system and trying to do so
    // will involve needing to derive and duplicate the CMake link order."
    // Instead, one can enable `MLIR_BUILD_MLIR_C_DYLIB` and link dynamically
    // with the libMLIR-C shared library as it containts everything.
    // See: https://github.com/llvm/llvm-project/blob/main/mlir/CMakeLists.txt#L122
    println!("cargo:rustc-link-search=all=llvm-project/build/lib");
    println!("cargo:rustc-link-lib=dylib=MLIR-C");

    // Generate and write out bindings to the MLIR C API.
    println!("cargo:rerun-if-changed=wrapper.h");
    let bindings = bindgen::Builder::default()
        .header("wrapper.h")
        .clang_arg("-Illvm-project/build/llvm-project/include")
        .parse_callbacks(Box::new(bindgen::CargoCallbacks))
        .generate()
        .expect("could not generate bindings.");
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("could not write bindings.");

    // Process the LALRPOP grammar file.
    lalrpop::Configuration::new()
        .log_info()
        .process_current_dir()
        .expect("could not run LALRPOP.");
}
