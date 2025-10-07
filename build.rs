fn main() {
    // Keep helpful error handling wiring (unchanged)
    linker_be_nice();

    // Keep the linker script arg
    println!("cargo:rustc-link-arg=-Tlinkall.x");

    if let Ok(libdir) = std::env::var("LIBBTDM_DIR") {
        // find the libbtdm*.a file in the dir
        let libfile = std::fs::read_dir(&libdir)
            .unwrap()
            .filter_map(|e| e.ok())
            .find_map(|entry| {
                let fname = entry.file_name().into_string().ok()?;
                if fname.starts_with("libbtdm") && fname.ends_with(".a") {
                    Some(fname)
                } else {
                    None
                }
            })
            .expect("no libbtdm*.a found in LIBBTDM_DIR");

        println!("cargo:warning=Using btdm archive: {} in {}", libfile, libdir);
        // add search path
        println!("cargo:rustc-link-search=native={}", libdir);

        // Force inclusion of all objects from the archive by filename.
        // Use -l:libname.a so we reference the exact file; wrap with whole-archive.
        println!("cargo:rustc-link-arg=-Wl,--whole-archive");
        println!("cargo:rustc-link-arg=-l:{}", libfile);
        println!("cargo:rustc-link-arg=-Wl,--no-whole-archive");
    } else {
        println!("cargo:warning=LIBBTDM_DIR not set; set it to the directory containing libbtdm*.a");
    }

    // Provide error-handling script arg (keeps helpful diagnostics)
    println!(
        "cargo:rustc-link-arg=-Wl,--error-handling-script={}",
        std::env::current_exe().unwrap().display()
    );
}

fn linker_be_nice() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() > 1 {
        let kind = &args[1];
        let what = &args[2];

        match kind.as_str() {
            "undefined-symbol" => match what.as_str() {
                "_defmt_timestamp" => {
                    eprintln!();
                    eprintln!("💡 `defmt` not found - make sure `defmt.x` is added as a linker script and you have included `use defmt_rtt as _;`");
                    eprintln!();
                }
                "_stack_start" => {
                    eprintln!();
                    eprintln!("💡 Is the linker script `linkall.x` missing?");
                    eprintln!();
                }
                "esp_wifi_preempt_enable"
                | "esp_wifi_preempt_yield_task"
                | "esp_wifi_preempt_task_create" => {
                    eprintln!();
                    eprintln!("💡 `esp-wifi` has no scheduler enabled. Make sure you have the `builtin-scheduler` feature enabled, or that you provide an external scheduler.");
                    eprintln!();
                }
                "embedded_test_linker_file_not_added_to_rustflags" => {
                    eprintln!();
                    eprintln!("💡 `embedded-test` not found - make sure `embedded-test.x` is added as a linker script for tests");
                    eprintln!();
                }
                _ => (),
            },
            _ => {
                std::process::exit(1);
            }
        }

        std::process::exit(0);
    }

    println!(
        "cargo:rustc-link-arg=-Wl,--error-handling-script={}",
        std::env::current_exe().unwrap().display()
    );
}