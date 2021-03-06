extern crate pkg_config;

use std::env;

fn main() {
    println!("openssl-sys-custom")
    let target = env::var("TARGET").unwrap();

    // libressl_pnacl_sys links the libs needed.
    if target.ends_with("nacl") { return; }

    let lib_dir = env::var("OPENSSL_LIB_DIR").ok();
    let include_dir = env::var("OPENSSL_INCLUDE_DIR").ok();

    if lib_dir.is_none() && include_dir.is_none() {
        // rustc doesn't seem to work with pkg-config's output in mingw64
        if !target.contains("windows") {

            let mut conf = ::pkg_config::Config::new();
            // Disable writing to STDOUT from pkg_config directly; we need to 
            // take a less clobbering approach
            conf.cargo_metadata(false);
            //Probe for openssl
            if let Ok(info) = conf.probe("openssl") {
                //We don't need to consider frameworks -F

                let ref pkg_libs = info.libs;
                let ref pkg_link_paths = info.link_paths;
                // handle (-l)
                for libname in pkg_libs {
                    // We asked pkg-config to include system lib folders, 
                    // So it's up to use to prevent clobbering
                    // rust link ordering is non-deterministic
                    for path in pkg_link_paths{
                        println!("cargo:rustc-link-search=native={}/lib{}*", path.to_str().unwrap(), libname);
                    }

                    //We aren't supporting static linking here
                    //but assuming this is OK
                    println!("cargo:rustc-link-lib={}", libname);

                }                   

                // handle (-I), avoid empty include paths as they are not supported by GCC
                if info.include_paths.len() > 0 {
                    let paths = env::join_paths(info.include_paths).unwrap();
                    println!("cargo:include={}", paths.to_str().unwrap());
                }
                return;
            }
        }
        if let Some(mingw_paths) = get_mingw_in_path() {
            for path in mingw_paths {
                println!("cargo:rustc-link-search=native={}", path);
            }
        }
    }

    let libs_env = env::var("OPENSSL_LIBS").ok();
    let libs = match libs_env {
        Some(ref v) => v.split(":").collect(),
        None => if target.contains("windows") {
            if get_mingw_in_path().is_some() && lib_dir.is_none() && include_dir.is_none() {
                vec!["ssleay32", "eay32"]
            } else {
                vec!["ssl32", "eay32"]
            }
        } else {
            vec!["ssl", "crypto"]
        }
    };

    let mode = if env::var_os("OPENSSL_STATIC").is_some() {
    	"static"
    } else {
    	"dylib"
    };

    if let Some(lib_dir) = lib_dir {
    	println!("cargo:rustc-link-search=native={}", lib_dir);
    }

    for lib in libs {
        println!("cargo:rustc-link-lib={}={}", mode, lib);
    }

    if let Some(include_dir) = include_dir {
        println!("cargo:include={}", include_dir);
    }
}

fn get_mingw_in_path() -> Option<Vec<String>> {
    match env::var_os("PATH") {
        Some(env_path) => {
            let paths: Vec<String> = env::split_paths(&env_path).filter_map(|path| {
                use std::ascii::AsciiExt;

                match path.to_str() {
                    Some(path_str) => {
                        if path_str.to_ascii_lowercase().contains("mingw") {
                            Some(path_str.to_string())
                        } else { None }
                    },
                    None => None
                }
            }).collect();

            if paths.len() > 0 { Some(paths) } else { None }
        },
        None => None
    }
}
