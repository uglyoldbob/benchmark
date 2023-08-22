#![allow(unused_imports, dead_code, unused_variables)]

#[cfg(feature = "bindgen")]
extern crate bindgen;
#[macro_use]
extern crate cfg_if;
#[cfg(feature = "bundled")]
extern crate cmake;
#[cfg(feature = "pkg-config")]
extern crate pkg_config;

use std::path::{Path, PathBuf};
use std::process::Command;
use std::{env, fs, io};

fn find_cargo_target_dir() -> PathBuf {
    // Infer the top level cargo target dir from the OUT_DIR by searching
    // upwards until we get to $CARGO_TARGET_DIR/build/ (which is always one
    // level up from the deepest directory containing our package name)
    let pkg_name = env::var("CARGO_PKG_NAME").unwrap();
    let mut out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    loop {
        {
            let final_path_segment = out_dir.file_name().unwrap();
            if final_path_segment.to_string_lossy().contains(&pkg_name) {
                break;
            }
        }
        if !out_dir.pop() {
            panic!("Malformed build path: {}", out_dir.to_string_lossy());
        }
    }
    out_dir.pop();
    out_dir.pop();
    out_dir
}

fn copy_library_file(src_path: &Path, target_path: &Path) {
    // Copy the shared libs to:
    //  * target dir: as a product ship product of the build,
    //  * deps directory: as comment example testing doesn't pick up the library search path
    //    otherwise and fails.
    let deps_path = target_path.join("deps");
    for path in &[target_path, &deps_path] {
        let dst_path = path.join(src_path.file_name().expect("Path missing filename"));

        fs::copy(&src_path, &dst_path).expect(&format!(
            "Failed to copy dynamic library from {} to {}",
            src_path.to_string_lossy(),
            dst_path.to_string_lossy()
        ));
    }
}

fn compile(sdl2_build_path: &Path, target_os: &str) -> PathBuf {
    let mut cfg = cmake::Config::new(sdl2_build_path);
    if let Ok(profile) = env::var("RAWDISK_BUILD_PROFILE") {
        cfg.profile(&profile);
    } else {
        cfg.profile("release");
    }

    // Allow specifying custom toolchain specifically for SDL2.
    if let Ok(toolchain) = env::var("RAWDISK_TOOLCHAIN") {
        cfg.define("CMAKE_TOOLCHAIN_FILE", &toolchain);
    } else {
        // Override __FLTUSED__ to keep the _fltused symbol from getting defined in the static build.
        // This conflicts and fails to link properly when building statically on Windows, likely due to
        // COMDAT conflicts/breakage happening somewhere.
        #[cfg(feature = "static-link")]
        cfg.cflag("-D__FLTUSED__");

        #[cfg(target_os = "linux")]
        {
            // Add common flag for affected version and above
            use version_compare::{compare_to, Cmp};
            if let Ok(version) = std::process::Command::new("cc")
                .arg("-dumpversion")
                .output()
            {
                let affected =
                    compare_to(std::str::from_utf8(&version.stdout).unwrap(), "10", Cmp::Ge)
                        .unwrap_or(true);
                if affected {
                    cfg.cflag("-fcommon");
                }
            }
        }
    }
    cfg.build()
}

fn main() {
    let target = env::var("TARGET").expect("Cargo build scripts always have TARGET");
    let host = env::var("HOST").expect("Cargo build scripts always have HOST");
    let target_os = get_os_from_triple(target.as_str()).unwrap();

    let source_path = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());

    let compiled_path: PathBuf = compile(source_path.as_path(), target_os);

    env::set_var("PKG_CONFIG_PATH", format!("{}", compiled_path.join("share").join("pkgconfig").display()));

    #[cfg(target_os = "linux")]
    let lib = pkg_config::Config::new()
        .atleast_version("0.1.0")
        .statik(true)
        .probe("rawdisk")
        .expect("Could not find a suitable version of rawdisk");
    #[cfg(target_os = "windows")]
    {
        println!("cargo:rustc-link-search=native={}", compiled_path.join("lib").display());
        println!("cargo:rustc-link-lib=rawdisk");
    }
}

fn get_os_from_triple(triple: &str) -> Option<&str> {
    triple.splitn(3, "-").nth(2)
}
