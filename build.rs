fn main() {
    linker_be_nice();

    // keep the expected linker script arg
    println!("cargo:rustc-link-arg=-Tlinkall.x");

    // Try to auto-detect the libbtdm*.a for the current target/profile
    let target = std::env::var("TARGET").unwrap_or_else(|_| "xtensa-esp32-none-elf".into());
    let profile = std::env::var("PROFILE").unwrap_or_else(|_| "release".into());
    let target_dir = std::env::var("CARGO_TARGET_DIR").unwrap_or_else(|_| "target".into());

    let mut found: Option<(String, String)> = None;

    // search under: <target_dir>/<target>/<profile>/build/*/out
    let search_base = std::path::Path::new(&target_dir)
        .join(&target)
        .join(&profile)
        .join("build");

    if let Ok(rd) = std::fs::read_dir(&search_base) {
        for entry in rd.filter_map(|e| e.ok()) {
            let out_dir = entry.path().join("out");
            if out_dir.is_dir() {
                if let Ok(out_rd) = std::fs::read_dir(&out_dir) {
                    if let Some(file) = out_rd
                        .filter_map(|e| e.ok())
                        .find_map(|ent| {
                            let f = ent.file_name().into_string().ok()?;
                            if f.starts_with("libbtdm") && f.ends_with(".a") {
                                Some(f)
                            } else {
                                None
                            }
                        })
                    {
                        found = Some((out_dir.display().to_string(), file));
                        break;
                    }
                }
            }
        }
    }

    // fallback to LIBBTDM_DIR env if auto-detect failed
    if found.is_none() {
        if let Ok(libdir) = std::env::var("LIBBTDM_DIR") {
            if let Ok(rd) = std::fs::read_dir(&libdir) {
                if let Some(file) = rd.filter_map(|e| e.ok()).find_map(|ent| {
                    let f = ent.file_name().into_string().ok()?;
                    if f.starts_with("libbtdm") && f.ends_with(".a") {
                        Some(f)
                    } else {
                        None
                    }
                }) {
                    found = Some((libdir, file));
                }
            }
        }
    }

    if let Some((libdir, libfile)) = found {
        println!("cargo:warning=Using btdm archive: {} in {}", libfile, libdir);
        println!("cargo:rustc-link-search=native={}", libdir);

        // force inclusion of all objects from the archive by filename
        println!("cargo:rustc-link-arg=-Wl,--whole-archive");
        println!("cargo:rustc-link-arg=-l:{}", libfile);
        println!("cargo:rustc-link-arg=-Wl,--no-whole-archive");
    } else {
        println!("cargo:warning=Could not find libbtdm*.a automatically; set LIBBTDM_DIR to the directory containing libbtdm*.a");
    }

    // keep helpful error handling wiring
    println!(
        "cargo:rustc-link-arg=-Wl,--error-handling-script={}",
        std::env::current_exe().unwrap().display()
    );
}

fn linker_be_nice() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() > 1 {
        // diagnostic helper invoked by the linker -- leave as no-op for normal builds
        std::process::exit(0);
    }

    // keep the diagnostic script wiring
    println!(
        "cargo:rustc-link-arg=-Wl,--error-handling-script={}",
        std::env::current_exe().unwrap().display()
    );
}