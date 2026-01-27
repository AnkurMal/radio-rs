use std::env;

fn main() {
    let target = env::var("CARGO_CFG_TARGET_OS").unwrap();

    let mut build = cc::Build::new();

    build
        .file("raudio/src/raudio.c")
        .include("raudio/src")
        .define("RAUDIO_IMPLEMENTATION", None)
        .define("RAUDIO_STANDALONE", None)
        .define("PLATFORM_DESKTOP", None);

    match target.as_str() {
        "windows" => {
            build.define("_CRT_SECURE_NO_WARNINGS", None);
            println!("cargo:rustc-link-lib=winmm");
            println!("cargo:rustc-link-lib=ole32");
        }
        "linux" => {
            println!("cargo:rustc-link-lib=asound");
            println!("cargo:rustc-link-lib=pthread");
            println!("cargo:rustc-link-lib=dl");
        }
        "macos" => {
            println!("cargo:rustc-link-lib=framework=CoreAudio");
            println!("cargo:rustc-link-lib=framework=AudioToolbox");
            println!("cargo:rustc-link-lib=framework=CoreFoundation");
        }
        _ => panic!("Unsupported platform"),
    }

    build.compile("audio");
}
