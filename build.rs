use std::env;
use std::path::PathBuf;

use cpp_build;

fn main() {
    let tf_include_dir = PathBuf::from(env::var("DEP_TENSORFLOW_LITE_INCLUDE").unwrap());
    println!("cargo:rustc-link-lib=dylib=edgetpu");

    cpp_build::Config::new()
        .include(tf_include_dir)
        .build("src/lib.rs");
}
