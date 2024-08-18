use std::error::Error;
use std::{env, path::PathBuf};

fn main() -> Result<(), Box<dyn Error>> {
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());

    tonic_build::configure()
        .build_server(true)
        .file_descriptor_set_path(out_dir.join("proto_descriptor.bin"))
        .compile(&["proto/finance_control.proto"], &["proto"])
        .unwrap();

    // trigger recompilation when a new migration is added
    println!("cargo:rerun-if-changed=migrations");

    Ok(())
}
