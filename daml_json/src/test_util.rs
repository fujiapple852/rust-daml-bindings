use once_cell::sync::OnceCell;

use daml_lf::element::DamlArchive;
use daml_lf::DarFile;

/// Load a dar and convert to an owned `DamlArchive`
pub fn daml_archive(path: &str) -> &'static DamlArchive<'static> {
    static INSTANCE: OnceCell<DamlArchive<'_>> = OnceCell::new();
    INSTANCE.get_or_init(|| {
        let dar = DarFile::from_file(path).unwrap_or_else(|_| panic!("dar file not found: {}", path));
        dar.to_owned_archive().expect("failed to convert dar to owned archive")
    })
}
