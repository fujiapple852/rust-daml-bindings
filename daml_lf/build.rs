use std::fs::File;
use std::io::{Error, Write};
use std::path::{Path, PathBuf};
use std::{env, error, fs};

const PROTOBUF_DAML_FILE: &str = "resources/protobuf/com/digitalasset/daml_lf_1_7/daml_lf.proto";
const PROTOBUF_PATH: &str = "resources/protobuf/";
const OUTPUT_PATH: &str = "src/protobuf_autogen";
const MODULE_HEADER: &str = "#![allow(clippy::all, clippy::pedantic, intra_doc_link_resolution_failure)]\n";

fn main() -> Result<(), Box<dyn error::Error>> {
    prost_build::compile_protos(&[PROTOBUF_DAML_FILE], &[PROTOBUF_PATH])?;
    let current_out_dir = env::var_os("OUT_DIR").ok_or_else(err_str)?.into_string().map_err(|_| err_str())?;
    fs::create_dir_all(OUTPUT_PATH)?;
    for file in fs::read_dir(current_out_dir)? {
        copy_entry(file?)?;
    }
    generate_module_src()?;
    println!("cargo:rerun-if-changed={}", PROTOBUF_PATH);
    Ok(())
}

fn copy_entry(file: fs::DirEntry) -> Result<(), Box<dyn error::Error>> {
    let file_path = file.path();
    let dest_file = file_path.file_name().ok_or_else(err_str)?.to_str().ok_or_else(err_str)?;
    fs::copy(&file_path, format!("{}/{}", OUTPUT_PATH, dest_file))?;
    Ok(())
}

fn generate_module_src() -> Result<(), Box<dyn error::Error>> {
    let all_generated_src = get_generated_files(OUTPUT_PATH)?;
    let names: Vec<String> = all_generated_src
        .iter()
        .map(|p| Ok(p.file_stem().ok_or("no filename")?.to_str().ok_or("invalid filename")?.to_owned()))
        .collect::<Result<Vec<String>, String>>()?;
    let mut file = File::create(Path::new(&format!("{}/mod.rs", OUTPUT_PATH)))?;
    file.write_all(MODULE_HEADER.as_bytes())?;
    for name in names {
        file.write_all(format!("pub mod {};\n", name).as_bytes())?;
    }
    Ok(())
}

fn get_generated_files<P: AsRef<Path>>(output_dir: P) -> Result<Vec<PathBuf>, Error> {
    fs::read_dir(output_dir)?
        .filter_map(|entry| match entry {
            Ok(d) => match d.path().file_stem() {
                Some(a) if a != "mod" => Some(Ok(d.path())),
                _ => None,
            },
            Err(e) => Some(Err(e)),
        })
        .collect()
}

fn err_str() -> &'static str {
    "error"
}
