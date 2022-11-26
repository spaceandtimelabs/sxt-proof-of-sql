fn main() {
    #[cfg(feature = "valgrind")]
    {
        println!("cargo:rerun-if-changed=src/toggle_collect.c");

        cc::Build::new()
            .file("src/toggle_collect.c")
            .compile("toggle_collect.a");
    }
}
