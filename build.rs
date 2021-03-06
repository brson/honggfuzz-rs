use std::env;
use std::process::Command;

const VERSION: &'static str = env!("CARGO_PKG_VERSION");

#[cfg(target_family="windows")]
compile_error!("honggfuzz-rs does not currently support Windows but works well under WSL (Windows Subsystem for Linux)");

#[cfg(not(any(target_os = "freebsd", target_os = "dragonfly", target_os = "bitrig", target_os = "openbsd", target_os = "netbsd")))]
const GNU_MAKE: &'static str = "make";
#[cfg(any(target_os = "freebsd", target_os = "dragonfly", target_os = "bitrig", target_os = "openbsd", target_os = "netbsd"))]
const GNU_MAKE: &'static str = "gmake";

fn main() {
    // Only build honggfuzz binaries if we are in the process of building an instrumentized binary
    let honggfuzz_target=  match env::var("CARGO_HONGGFUZZ_TARGET_DIR") {
        Ok(path) => path, // path where to place honggfuzz binary. provided by cargo-hfuzz command.
        Err(_) => return
    };

    // check that "cargo hfuzz" command is at the same version as this file
    let honggfuzz_build_version = env::var("CARGO_HONGGFUZZ_BUILD_VERSION").unwrap_or("unknown".to_string());
    assert!(VERSION == honggfuzz_build_version,
            "hongfuzz dependency ({}) and build command ({}) versions do not match",
            VERSION, honggfuzz_build_version);

    // retrieve env variable provided by cargo
    let out_dir = env::var("OUT_DIR").unwrap();
    let pwd = env::var("PWD").unwrap();

    // clean upsteam honggfuzz directory
    let status = Command::new(GNU_MAKE)
        .args(&["-C", "honggfuzz", "clean"])
        .status()
        .expect("failed to run \"make -C honggfuzz clean\"");
    assert!(status.success());

    // build honggfuzz command and hfuzz static library
    let status = Command::new(GNU_MAKE)
        .args(&["-C", "honggfuzz", "honggfuzz", "libhfuzz/libhfuzz.a"])
        .status()
        .expect("failed to run \"make -C honggfuzz hongfuzz libhfuzz/libhfuzz.a\"");
    assert!(status.success());

    // copy hfuzz static library to output directory
    let status = Command::new("cp")
        .args(&["honggfuzz/libhfuzz/libhfuzz.a", &out_dir])
        .status()
        .expect(&format!("failed to run \"cp honggfuzz/libhfuzz/libhfuzz.a {}\"", &out_dir));
    assert!(status.success());

    // copy honggfuzz executable to honggfuzz target directory
    let status = Command::new("cp")
        .args(&["honggfuzz/honggfuzz", &format!("{}/{}", &pwd, &honggfuzz_target)])
        .status()
        .expect(&format!("failed to run \"cp honggfuzz/honggfuzz {}\"", &honggfuzz_target));
    assert!(status.success());

    // tell cargo how to link final executable to hfuzz static library
    println!("cargo:rustc-link-lib=static={}", "hfuzz");
    println!("cargo:rustc-link-search=native={}", &out_dir);
}
