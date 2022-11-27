#![allow(clippy::needless_borrow)]

use std::path::Path;

fn protobuf() -> anyhow::Result<()> {
    let protobuf = Path::new("protobuf").to_owned();

    let input: Vec<_> = protobuf
        .read_dir()
        .expect("protobuf directory")
        .filter_map(|entry| {
            let entry = entry.expect("readable protobuf directory");
            let path = entry.path();
            if Some("proto") == path.extension().and_then(std::ffi::OsStr::to_str) {
                assert!(path.is_file());
                println!("cargo:rerun-if-changed={}", path.to_str().unwrap());
                Some(path)
            } else {
                None
            }
        })
        .collect();

    prost_build::compile_protos(&input, &[protobuf])?;
    Ok(())
}

fn main() {
    protobuf().unwrap();

    let macos_lib_search = if cfg!(target_os = "macos") {
        "=framework"
    } else {
        ""
    };

    let sailfish_libs: &[&str] = if cfg!(feature = "sailfish") {
        &["qt5embedwidget"]
    } else {
        &[]
    };
    let libs = ["dbus-1"];
    for lib in libs.iter().chain(sailfish_libs.iter()) {
        println!("cargo:rustc-link-lib{}={}", macos_lib_search, lib);
    }
}
