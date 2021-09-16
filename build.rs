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
use std::io::prelude::*;
use std::io::BufReader;
use std::path::Path;
use std::process::Command;

use failure::*;
use vergen::*;

fn qmake_query(var: &str) -> String {
    let qmake = std::env::var("QMAKE").unwrap_or_else(|_| "qmake".to_string());
    String::from_utf8(
        Command::new(qmake)
            .env("QT_SELECT", "qt5")
            .args(&["-query", var])
            .output()
            .expect("Failed to execute qmake. Make sure 'qmake' is in your path")
            .stdout,
    )
    .expect("UTF-8 conversion failed")
}

fn detect_qt_version(qt_include_path: &Path) -> Result<String, Error> {
    let path = qt_include_path.join("QtCore").join("qconfig.h");
    let f = std::fs::File::open(&path).unwrap_or_else(|_| panic!("Cannot open `{:?}`", path));
    let b = BufReader::new(f);

    // append qconfig-64.h or config-32.h, depending on TARGET_POINTER_WIDTH
    let arch_specific: Box<dyn BufRead> = {
        let pointer_width = std::env::var("CARGO_CFG_TARGET_POINTER_WIDTH").unwrap();
        let path = qt_include_path
            .join("QtCore")
            .join(format!("qconfig-{}.h", pointer_width));
        match std::fs::File::open(&path) {
            Ok(f) => Box::new(BufReader::new(f)),
            Err(_) => Box::new(std::io::Cursor::new("")),
        }
    };

    let regex = regex::Regex::new("#define +QT_VERSION_STR +\"(.*)\"")?;

    for line in b.lines().chain(arch_specific.lines()) {
        let line = line.expect("qconfig.h is valid UTF-8");
        if let Some(capture) = regex.captures_iter(&line).next() {
            return Ok(capture[1].into());
        }
        if line.contains("QT_VERION_STR") {
            bail!("QT_VERSION_STR: {}, not matched by regex", line);
        }
    }
    bail!("Could not detect Qt version");
}

#[cfg(feature = "bundled-sqlcipher")]
// static sqlcipher handling. Needed for compatibility with
// sailfish-components-webview.
// This may become obsolete with an sqlcipher upgrade from jolla or when
// https://gitlab.com/rubdos/whisperfish/-/issues/227 is implemented.
fn build_sqlcipher() {
    // `cc` currently does not ship incremental compilation:
    // https://github.com/alexcrichton/cc-rs/issues/230
    let before = std::fs::metadata("sqlcipher/sqlite3.c")
        .map(|x| x.modified().unwrap())
        .ok();

    // Download and prepare sqlcipher source
    let stat = Command::new("sqlcipher/get-sqlcipher.sh")
        .status()
        .expect("Failed to download sqlcipher");
    assert!(stat.success());

    let after = std::fs::metadata("sqlcipher/sqlite3.c")
        .map(|x| x.modified().unwrap())
        .unwrap();

    // If sqlite3.c changed, we recompile
    let exists = std::path::PathBuf::from(std::env::var("OUT_DIR").unwrap())
        .join("libsqlcipher.a")
        .is_file();
    let needs_rerun = !exists || before.map(|before| before != after).unwrap_or(true);

    if needs_rerun {
        // Build static sqlcipher
        cc::Build::new()
            .file("sqlcipher/sqlite3.c")
            .warnings(false)
            .include("/usr/include/openssl/")
            .flag("-Wno-stringop-overflow")
            .flag("-Wno-return-local-addr")
            .flag("-DSQLITE_CORE")
            .flag("-DSQLITE_DEFAULT_FOREIGN_KEYS=1")
            .flag("-DSQLITE_ENABLE_API_ARMOR")
            .flag("-DSQLITE_HAS_CODEC")
            .flag("-DSQLITE_TEMP_STORE=2")
            .flag("-DHAVE_ISNAN")
            .flag("-DHAVE_LOCALTIME_R")
            .flag("-DSQLITE_ENABLE_COLUMN_METADATA")
            .flag("-DSQLITE_ENABLE_DBSTAT_VTAB")
            .flag("-DSQLITE_ENABLE_FTS3")
            .flag("-DSQLITE_ENABLE_FTS3_PARENTHESIS")
            .flag("-DSQLITE_ENABLE_FTS5")
            .flag("-DSQLITE_ENABLE_JSON1")
            .flag("-DSQLITE_ENABLE_LOAD_EXTENSION=1")
            .flag("-DSQLITE_ENABLE_MEMORY_MANAGEMENT")
            .flag("-DSQLITE_ENABLE_RTREE")
            .flag("-DSQLITE_ENABLE_STAT2")
            .flag("-DSQLITE_ENABLE_STAT4")
            .flag("-DSQLITE_SOUNDEX")
            .flag("-DSQLITE_THREADSAFE=1")
            .flag("-DSQLITE_USE_URI")
            .flag("-DHAVE_USLEEP=1")
            .compile("sqlcipher");
    } else {
        println!("cargo:lib_dir={}", std::env::var("OUT_DIR").unwrap());
        println!("cargo:rustc-link-lib=static=sqlcipher");
        println!("cargo:rerun-if-changed=sqlcipher/sqlite3.c");
        println!("cargo:rerun-if-changed=sqlcipher/get-sqlcipher.sh");
    }
}

fn protobuf() -> Result<(), Error> {
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

    // Print a warning when rustc is too old.
    if !version_check::is_min_version("1.48.0").unwrap_or(false) {
        if let Some(version) = version_check::Version::read() {
            panic!(
                "Whisperfish requires Rust 1.48.0 or later.  You are using rustc {}",
                version
            );
        } else {
            panic!(
                "Whisperfish requires Rust 1.48.0 or later, but could not determine Rust version.",
            );
        }
    }

    let qt_include_path = qmake_query("QT_INSTALL_HEADERS");
    let qt_include_path = qt_include_path.trim();

    let mut cfg = cpp_build::Config::new();

    let qt_version = detect_qt_version(std::path::Path::new(&qt_include_path)).unwrap();
    cfg.include(format!("{}/QtGui/{}", qt_include_path, qt_version));

    // This is kinda hacky. Sorry.
    cfg.include(&qt_include_path)
        .include("/usr/include/sailfishapp/")
        .include(format!("{}/QtCore", qt_include_path))
        // -W deprecated-copy triggers some warnings in old Jolla's Qt distribution.
        // It is annoying to look at while developing, and we cannot do anything about it
        // ourselves.
        .flag("-Wno-deprecated-copy")
        .build("src/lib.rs");

    let contains_cpp = [
        "qmlapp/mod.rs",
        "qmlapp/tokio_qt.rs",
        "qmlapp/native.rs",
        "config/settings.rs",
    ];
    for f in &contains_cpp {
        println!("cargo:rerun-if-changed=src/{}", f);
    }

    let macos_lib_search = if cfg!(target_os = "macos") {
        "=framework"
    } else {
        ""
    };
    let macos_lib_framework = if cfg!(target_os = "macos") { "" } else { "5" };

    let qt_libs = ["OpenGL", "Gui", "Core", "Quick", "Qml"];
    for lib in &qt_libs {
        println!(
            "cargo:rustc-link-lib{}=Qt{}{}",
            macos_lib_search, macos_lib_framework, lib
        );
    }

    let sailfish_libs: &[&str] = if cfg!(feature = "sailfish") {
        &["sailfishapp", "qt5embedwidget"]
    } else {
        &[]
    };
    let libs = ["EGL", "dbus-1"];
    for lib in libs.iter().chain(sailfish_libs.iter()) {
        println!("cargo:rustc-link-lib{}={}", macos_lib_search, lib);
    }

    #[cfg(feature = "bundled-sqlcipher")]
    build_sqlcipher();

    // vergen
    let mut cfg = vergen::Config::default();
    *cfg.git_mut().enabled_mut() = true;
    *cfg.git_mut().sha_mut() = true;
    *cfg.git_mut().sha_kind_mut() = vergen::ShaKind::Short;
    vergen(cfg).expect("vergen setup");
}
