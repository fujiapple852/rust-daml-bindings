use itertools::Itertools;
use std::error;
use std::fs;
use std::io::Error;
use std::path::Path;
use std::path::PathBuf;

const ALL_PROTO_SRC_PATHS: &[&str] = &[
    "com/daml/ledger/api/v1",
    "com/daml/ledger/api/v1/testing",
    "com/daml/ledger/api/v1/admin",
    "google/protobuf",
    "google/rpc",
];
const PROTO_ROOT_PATH: &str = "resources/protobuf";

fn main() -> Result<(), Box<dyn error::Error>> {
    let all_protos = get_all_protos(ALL_PROTO_SRC_PATHS)?;
    tonic_build::configure()
        .build_server(false)
        .build_client(true)
        .format(true)
        .compile(all_protos.as_slice(), vec![<str as AsRef<Path>>::as_ref(PROTO_ROOT_PATH)].as_slice())?;
    Ok(())
}

fn get_all_protos(src_paths: &[&str]) -> Result<Vec<PathBuf>, Error> {
    src_paths.iter().map(Path::new).map(get_protos_from_dir).fold_ok(vec![], |mut acc: Vec<PathBuf>, v| {
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
