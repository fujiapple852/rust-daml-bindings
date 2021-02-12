use crate::archive::DamlLfArchive;
use crate::convert;
use crate::element::DamlArchive;
use crate::error::{DamlLfError, DamlLfResult};
use crate::manifest::DarManifest;
use crate::DEFAULT_ARCHIVE_NAME;
use std::ffi::OsStr;
use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};
use zip::ZipArchive;

const MANIFEST_FILE_PATH: &str = "META-INF/MANIFEST.MF";
const DALF_FILE_EXTENSION: &str = "dalf";
const DALF_PRIM_FILE_SUFFIX: &str = "-prim";

/// A collection of `DAML LF` archives combined with a manifest file (aka a `dar` file).
///
/// A `DarFile` contains a `main` [`DamlLfArchive`] and collection of `dependencies` [`DamlLfArchive`] combined with
/// a [`DarManifest`].
#[derive(Debug, Clone)]
pub struct DarFile {
    pub manifest: DarManifest,
    pub main: DamlLfArchive,
    pub dependencies: Vec<DamlLfArchive>,
}

impl DarFile {
    /// Create a new `DarFile` from an existing `manifest` file, main and `dependencies` [`DamlLfArchive`].
    ///
    /// Note that this method does not validate that the supplied `manifest` correctly reflects the `main` and
    /// `dependencies` [`DamlLfArchive`] provided and so may yield an invalid `DarFile`.
    pub fn new(
        manifest: impl Into<DarManifest>,
        main: impl Into<DamlLfArchive>,
        dependencies: impl Into<Vec<DamlLfArchive>>,
    ) -> Self {
        Self {
            manifest: manifest.into(),
            main: main.into(),
            dependencies: dependencies.into(),
        }
    }

    /// Create a `DarFile` from the supplied `dar` file.
    ///
    /// There are currently two supported `dar` formats supported by this module, `legacy` and `fat`.  Parsing will
    /// first attempt to parse a `fat` `dar`.  If parsing fails an attempt will be made to parse a `legacy` `dar`
    /// instead.
    ///
    /// # Dar Format
    ///
    /// Both formats are compressed zip archives with a `dar` extension which contain a `META-INF/MANIFEST.MF` file and
    /// one or more `dalf` files, potentially nested in a sub folders.
    ///
    /// If `dar` file provided does not contain a manifest or if the manifest does not contain all mandatory fields then
    /// parsing will fail.
    ///
    /// The manifest file of `legacy` `dar` files will not be read, instead it will be inferred from the set of `dalf`
    /// files within the file.  The following combinations are considered valid for `legacy` `dar` files:
    ///
    /// - A `dar` file containing only a single non-prim `dalf` file (anywhere)
    /// - A `dar` file containing a single non-prim `dalf` file and a single prim `dalf` file (ending with the `-prim`
    /// suffix)
    /// - A `dar` file containing only a single prim (ending with the `-prim` suffix) file
    ///
    /// # Errors
    ///
    /// If the file cannot be read then an [`IOError`] will be returned.
    ///
    /// If the file cannot be interpreted as a `zip` archive then a [`DarParseError`] will be returned.
    ///
    /// Should both `fat` and `legacy` parsing attempts fail then a [`DarParseError`] will be returned.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use daml_lf::DarFile;
    /// # use daml_lf::DamlLfResult;
    /// # use daml_lf::DamlLfHashFunction;
    /// # fn main() -> DamlLfResult<()> {
    /// let dar = DarFile::from_file("Example.dar")?;
    /// assert_eq!(&DamlLfHashFunction::SHA256, dar.main().hash_function());
    /// # Ok(())
    /// # }
    /// ```
    /// [`IOError`]: DamlLfError::IOError
    /// [`DarParseError`]: DamlLfError::DarParseError
    pub fn from_file(path: impl AsRef<Path>) -> DamlLfResult<Self> {
        let dar_file = std::fs::File::open(path)?;
        let mut zip_archive = zip::ZipArchive::new(dar_file)?;
        let manifest = match Self::parse_dar_manifest_from_file(&mut zip_archive) {
            Ok(manifest) => Ok(manifest),
            Err(_) => Self::make_manifest_from_archive(&mut zip_archive),
        }?;
        let dalf_main = Self::parse_dalf_from_archive(&mut zip_archive, manifest.dalf_main())?;
        let dalf_dependencies = Self::parse_dalfs_from_archive(&mut zip_archive, manifest.dalf_dependencies())?;
        Ok(Self::new(manifest, dalf_main, dalf_dependencies))
    }

    /// Create a [`DamlArchive`] from this [`DarFile`] and apply it to `f`.
    ///
    /// The created [`DamlArchive`] borrows all interned string data from this [`DarFile`] and is therefore tied to the
    /// lifetime of the [`DarFile`] and so cannot be returned from this scope.  The [`DamlArchive`] can be accessed from
    /// the supplied closure `f` which may return owned data.
    ///
    /// Use [`DarFile::to_owned_archive`] to create a [`DamlArchive`] which does not borrow any data from the generating
    /// [`DarFile`].
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use daml_lf::DarFile;
    /// # use daml_lf::DamlLfResult;
    /// # fn main() -> DamlLfResult<()> {
    /// let dar = DarFile::from_file("Example.dar")?;
    /// // create a DamlArchive from this DarFile and extract the (owned) name.
    /// let name = dar.apply(|archive| archive.name().to_owned())?;
    /// assert_eq!("Example-1.0.0", name);
    /// Ok(())
    /// # }
    /// ```
    pub fn apply<R, F>(&self, f: F) -> DamlLfResult<R>
    where
        F: FnOnce(&DamlArchive<'_>) -> R,
    {
        convert::apply_dar(self, f)
    }

    /// Create an owned [`DamlArchive`] from this [`DarFile`].
    ///
    /// This is an expensive operation as it involves both a conversion of the [`DarFile`] to a [`DamlArchive`] (which
    /// borrows all interned strings) and a subsequent conversion to an owned [`DamlArchive`] which clones all interned
    /// strings.
    ///
    /// Use this when an owned instance of a [`DamlArchive`] is required, such as for passing to a thread.  For other
    /// cases consider using the [`DarFile::apply`] method which does not require the second conversion.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use daml_lf::DarFile;
    /// # use daml_lf::DamlLfResult;
    /// # fn main() -> DamlLfResult<()> {
    /// let dar = DarFile::from_file("Example.dar")?;
    /// let archive = dar.to_owned_archive()?;
    /// assert_eq!("TestingTypes-1.0.0", archive.name());
    /// # Ok(())
    /// # }
    /// ```
    pub fn to_owned_archive(&self) -> DamlLfResult<DamlArchive<'static>> {
        convert::to_owned_archive(self)
    }

    /// The `manifest` information contained within this `DarFile`.
    pub const fn manifest(&self) -> &DarManifest {
        &self.manifest
    }

    /// The `main` [`DamlLfArchive`] contained within this `DarFile`.
    pub const fn main(&self) -> &DamlLfArchive {
        &self.main
    }

    /// A collection of `dependencies` [`DamlLfArchive`] contained within this `DarFile`.
    pub const fn dependencies(&self) -> &Vec<DamlLfArchive> {
        &self.dependencies
    }

    fn is_dalf(path: &Path) -> bool {
        path.extension().and_then(OsStr::to_str).map(str::to_lowercase).map_or(false, |q| q == DALF_FILE_EXTENSION)
    }

    fn is_prim_dalf(path: &Path) -> bool {
        path.file_stem()
            .and_then(OsStr::to_str)
            .map(str::to_lowercase)
            .map_or(false, |p| p.ends_with(DALF_PRIM_FILE_SUFFIX))
    }

    fn make_manifest_from_archive(zip_archive: &mut ZipArchive<File>) -> DamlLfResult<DarManifest> {
        let dalf_paths = zip_archive.paths();
        let (prim, main): (Vec<PathBuf>, Vec<PathBuf>) =
            dalf_paths.into_iter().filter(|d| Self::is_dalf(d)).partition(|d| Self::is_prim_dalf(d));
        let (dalf_main_path, dalf_dependencies_paths) = match (prim.as_slice(), main.as_slice()) {
            ([p], [m]) => Ok((p, vec![m])),
            ([p], []) => Ok((p, vec![])),
            ([], [m]) => Ok((m, vec![])),
            _ => Err(DamlLfError::new_dar_parse_error("invalid legacy Dar")),
        }?;

        let manifest = DarManifest::new_implied(
            dalf_main_path.display().to_string(),
            dalf_dependencies_paths.into_iter().map(|d| d.display().to_string()).collect(),
        );
        Ok(manifest)
    }

    fn parse_dalfs_from_archive(
        zip_archive: &mut ZipArchive<File>,
        paths: &[String],
    ) -> DamlLfResult<Vec<DamlLfArchive>> {
        paths
            .iter()
            .map(|dalf_path| Self::parse_dalf_from_archive(zip_archive, dalf_path))
            .collect::<DamlLfResult<Vec<DamlLfArchive>>>()
    }

    #[allow(clippy::cast_possible_truncation)]
    fn parse_dalf_from_archive(zip_archive: &mut ZipArchive<File>, location: &str) -> DamlLfResult<DamlLfArchive> {
        let mut file = zip_archive.by_name(location)?;
        let mut buf = Vec::with_capacity(file.size() as usize);
        file.read_to_end(&mut buf)?;
        let archive_name_buffer = PathBuf::from(location);
        let archive_name_stem = archive_name_buffer.file_stem().and_then(OsStr::to_str).unwrap_or(DEFAULT_ARCHIVE_NAME);
        DamlLfArchive::from_bytes_named(archive_name_stem, buf)
    }

    fn parse_dar_manifest_from_file(zip_archive: &mut ZipArchive<File>) -> DamlLfResult<DarManifest> {
        let mut file = zip_archive.by_name(MANIFEST_FILE_PATH)?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;
        DarManifest::parse(&contents)
    }
}

trait ZipArchiveEx<T> {
    fn paths(&mut self) -> Vec<PathBuf>;
    fn contains(&mut self, path: &str) -> bool;
}

impl ZipArchiveEx<File> for ZipArchive<File> {
    fn paths(&mut self) -> Vec<PathBuf> {
        let mut paths = Vec::with_capacity(self.len());
        for i in 0..self.len() {
            if let Ok(file) = self.by_index(i) {
                paths.push(PathBuf::from(file.name()))
            }
        }
        paths
    }

    fn contains(&mut self, path: &str) -> bool {
        self.by_name(path).is_ok()
    }
}
