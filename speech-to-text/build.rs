fn main() {
    //println!("cargo:rustc-link-search=native=vosk/vosk-api-0.3.50");
    println!("cargo:rustc-link-search=native=vosk/vosk-osx-0.3.42");
    println!("cargo:rustc-link-lib=dylib=vosk");
}