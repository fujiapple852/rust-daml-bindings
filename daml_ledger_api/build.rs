use itertools::Itertools;
use std::error;
use std::fs;
use std::fs::File;
use std::io::Error;
use std::io::Write;
use std::path::Path;
use std::path::PathBuf;

const ALL_PROTO_SRC_PATHS: &[&str] =
    &["com/digitalasset/ledger/api/v1", "com/digitalasset/ledger/api/v1/testing", "google/protobuf", "google/rpc"];
const PROTO_ROOT_PATH: &str = "resources/protobuf";
const OUTPUT_PATH: &str = "src/grpc_protobuf_autogen";
const MODULE_HEADER: &str = "#![allow(clippy::all, clippy::pedantic)]\n#![allow(renamed_and_removed_lints)]\n";

fn main() -> Result<(), Box<dyn error::Error>> {
    fs::create_dir_all(OUTPUT_PATH)?;
    let all_protos = get_all_protos(ALL_PROTO_SRC_PATHS)?;
    protoc_grpcio::compile_grpc_protos(all_protos, &[PROTO_ROOT_PATH], OUTPUT_PATH, None)?;
    generate_module_src()?;
    println!("cargo:rerun-if-changed={}", PROTO_ROOT_PATH);
    Ok(())
}

fn generate_module_src() -> Result<(), Box<dyn error::Error>> {
    let all_generated_src = get_generated_files(OUTPUT_PATH)?;
    let names: Vec<String> = all_generated_src
        .iter()
        .map(|p| Ok(p.file_stem().ok_or("no filename")?.to_str().ok_or("invalid filename")?.to_owned()))
        .collect::<Result<Vec<String>, String>>()?;
    let mut file = File::create(Path::new(&format!("{}/mod.rs", OUTPUT_PATH)))?;
    file.write(MODULE_HEADER.as_bytes())?;
    for name in names {
        file.write(format!("pub mod {};\n", name).as_bytes())?;
    }
    Ok(())
}

fn get_all_protos(src_paths: &[&str]) -> Result<Vec<PathBuf>, Error> {
    src_paths.iter().map(Path::new).map(get_protos_from_dir).fold_results(vec![], |mut acc: Vec<PathBuf>, v| {
        acc.extend(v);
        acc
    })
}

fn get_protos_from_dir(dir: &Path) -> Result<Vec<PathBuf>, Error> {
    fs::read_dir(Path::new(PROTO_ROOT_PATH).join(dir))?
        .filter_map(|entry| match entry {
            Ok(d) => match d.path().extension() {
                Some(a) if a == "proto" => Some(Ok(d.path())),
                _ => None,
            },
            Err(e) => Some(Err(e)),
        })
        .collect()
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
