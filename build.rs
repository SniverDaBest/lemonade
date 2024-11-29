fn main() {
    // Specify the directory where your C source files are located
    let out_dir = std::env::var("OUT_DIR").unwrap();

    // Use cc crate to compile the C code
    cc::Build::new().file("src/cmos.c").compile("lemonade_cmos");

    println!("cargo:rerun-if-changed=src/cmos.c");
}
