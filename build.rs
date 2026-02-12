//! Set up linker scripts for the rp235x-hal examples

use std::fs::File;
use std::io::Write;
use std::path::PathBuf;

fn main() {
    // Put the linker script somewhere the linker can find it
    let out = PathBuf::from(std::env::var_os("OUT_DIR").unwrap());
    println!("cargo:rustc-link-search={}", out.display());
    download_cyw43_firmware();

    // The file `memory.x` is loaded by cortex-m-rt's `link.x` script, which
    // is what we specify in `.cargo/config.toml` for Arm builds
    let memory_x = include_bytes!("memory.x");
    let mut f = File::create(out.join("memory.x")).unwrap();
    f.write_all(memory_x).unwrap();
    println!("cargo:rerun-if-changed=memory.x");

    // The file `rp235x_riscv.x` is what we specify in `.cargo/config.toml` for
    // RISC-V builds
    let rp235x_riscv_x = include_bytes!("rp235x_riscv.x");
    let mut f = File::create(out.join("rp235x_riscv.x")).unwrap();
    f.write_all(rp235x_riscv_x).unwrap();
    println!("cargo:rerun-if-changed=rp235x_riscv.x");

    println!("cargo:rerun-if-changed=build.rs");
}

fn download_cyw43_firmware() {
    let download_folder = "firmware";
    let url_base = "https://github.com/embassy-rs/embassy/raw/refs/heads/main/cyw43-firmware";
    let file_names = [
        "43439A0.bin",
        "43439A0_btfw.bin",
        "43439A0_clm.bin",
        "LICENSE-permissive-binary-license-1.0.txt",
        "README.md",
    ];

    println!("cargo::rerun-if-changed=build.rs");
    println!("cargo::rerun-if-changed={}", download_folder);
    std::fs::create_dir_all(download_folder).expect("Failed to create download directory");

    // download each file into the folder "cyw43-firmware"
    for file in file_names {
        let url = format!("{}/{}", url_base, file);
        // only fetch if it doesn't exist
        if std::path::Path::new(download_folder).join(file).exists() {
            continue;
        }
        match reqwest::blocking::get(&url) {
            Ok(response) => {
                let content = response.bytes().expect("Failed to read file content");
                let file_path = PathBuf::from(download_folder).join(file);
                std::fs::write(file_path, &content).expect("Failed to write file");
            }
            Err(err) => panic!(
                "Failed to download the cyw43 firmware from {}: {}, required for pi-pico-w example",
                url, err
            ),
        }
    }
}
