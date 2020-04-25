/* Copyright (C) 2018 Olivier Goffart <ogoffart@woboq.com>
Permission is hereby granted, free of charge, to any person obtaining a copy of this software and
associated documentation files (the "Software"), to deal in the Software without restriction,
including without limitation the rights to use, copy, modify, merge, publish, distribute, sublicense,
and/or sell copies of the Software, and to permit persons to whom the Software is furnished to do so,
subject to the following conditions:
The above copyright notice and this permission notice shall be included in all copies or substantial
portions of the Software.
THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR IMPLIED, INCLUDING BUT
NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND
NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES
OR OTHER LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR IN
CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE SOFTWARE.
*/
use std::env;
use std::path::Path;
use std::io::Write;

use failure::*;

fn mock_libc(mer_root: &str, arch: &str) -> Result<String, Error> {
    let out_dir = env::var("OUT_DIR")?;
    let qml_path = &Path::new(&out_dir).join("libc.so");

    let mut f = std::fs::File::create(qml_path)?;
    match arch {
        "armv7hl" => {
            writeln!(f, "OUTPUT_FORMAT(elf32-littlearm)")?;
            writeln!(f, "GROUP ( {}/lib/libc.so.6 {}/usr/lib/libc_nonshared.a  AS_NEEDED ( {}/lib/ld-linux-armhf.so.3 ))",
                mer_root, mer_root, mer_root)?;
        },
        "i486" => {
            writeln!(f, "OUTPUT_FORMAT(elf32-i386)")?;
            writeln!(f, "GROUP ( {}/lib/libc.so.6 {}/usr/lib/libc_nonshared.a  AS_NEEDED ( {}/lib/ld-linux.so.2 ))",
                mer_root, mer_root, mer_root)?;
        },
        _ => unreachable!(),
    }

    Ok(out_dir)
}

fn qml_to_qrc() -> Result<(), Error> {
    let out_dir = &env::var("OUT_DIR")?;
    let qml_path = &Path::new(&out_dir).join("qml.rs");

    let mut f = std::fs::File::create(qml_path)?;

    let mut read_dirs = std::collections::VecDeque::new();
    read_dirs.push_back(std::fs::read_dir("qml")?);

    writeln!(f, "qrc!{{qml_resources, \"qml\" {{ ")?;

    while let Some(read_dir) = read_dirs.pop_front() {
        for entry in read_dir {
            let entry = entry?.path();
            if entry.is_dir() {
                read_dirs.push_back(std::fs::read_dir(entry)?);
            } else if entry.is_file() {
                println!("cargo:rerun-if-changed={}", entry.display());
                writeln!(f, "{:?},", entry)?;
            }
        }
    }

    writeln!(f, " }} }}")?;

    Ok(())
}

fn main() {
    let mer_sdk = std::env::var("MERSDK").ok()
        .expect("MERSDK should be set.");

    let mer_target = std::env::var("MER_TARGET").ok().unwrap_or("SailfishOS-latest".into());

    let arch = match &std::env::var("CARGO_CFG_TARGET_ARCH").unwrap() as &str {
        "arm" => "armv7hl",
        "i686" => "i486",
        "x86" => "i486",
        unsupported => panic!("Target {} is not supported for Mer", unsupported),
    };

    let mer_target_root = format!("{}/targets/{}-{}", mer_sdk, mer_target, arch);

    cpp_build::Config::new()
        .include(format!("{}/usr/include/", mer_target_root))
        .include(format!("{}/usr/include/sailfishapp/", mer_target_root))
        .include(format!("{}/usr/include/qt5/", mer_target_root))
        .include(format!("{}/usr/include/qt5/QtCore", mer_target_root))
        .build("src/sfos.rs");

    println!("cargo:rerun-if-changed=src/sfos.rs");

    let macos_lib_search = if cfg!(target_os = "macos") {
        "=framework"
    } else {
        ""
    };
    let macos_lib_framework = if cfg!(target_os = "macos") { "" } else { "5" };

    let mock_lib_path = mock_libc(&mer_target_root, arch).unwrap();
    println!(
        "cargo:rustc-link-search{}={}",
        macos_lib_search,
        mock_lib_path,
    );

    println!("cargo:rustc-bin-link-arg=-rpath-link,{}/usr/lib", mer_target_root);
    println!("cargo:rustc-bin-link-arg=-rpath-link,{}/lib", mer_target_root);

    println!(
        "cargo:rustc-link-search{}={}/toolings/{}/opt/cross/{}-meego-linux-gnueabi/lib",
        macos_lib_search,
        mer_sdk, mer_target, arch
    );

    println!(
        "cargo:rustc-link-search{}={}/usr/lib/qt5/qml/Nemo/Notifications/",
        macos_lib_search,
        mer_target_root
    );

    println!(
        "cargo:rustc-link-search{}={}/toolings/{}/opt/cross/lib/gcc/{}-meego-linux-gnueabi/4.9.4/",
        macos_lib_search,
        mer_sdk, mer_target, arch
    );

    println!(
        "cargo:rustc-link-search{}={}/usr/lib/",
        macos_lib_search,
        mer_target_root,
    );

    let qt_libs = ["OpenGL", "Gui", "Core", "Quick", "Qml"];
    for lib in &qt_libs {
        println!("cargo:rustc-link-lib{}=Qt{}{}", macos_lib_search, macos_lib_framework, lib);
    }

    let libs = ["EGL", "nemonotifications", "sailfishapp"];
    for lib in &libs {
        println!("cargo:rustc-link-lib{}={}", macos_lib_search, lib);
    }

    qml_to_qrc().unwrap();
}
