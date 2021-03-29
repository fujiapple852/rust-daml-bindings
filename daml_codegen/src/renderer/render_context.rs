use daml_lf::element::DamlArchive;

/// Contextual information required during code generation.
#[derive(Debug)]
pub struct RenderContext<'a> {
    mode: RenderMode<'a>,
    filter_mode: RenderFilterMode,
}

impl Default for RenderContext<'_> {
    fn default() -> Self {
        Self {
            mode: RenderMode::default(),
            filter_mode: RenderFilterMode::default(),
        }
    }
}

impl<'a> RenderContext<'a> {
    /// Create a new `RenderContext` with a `DamlArchive`.
    pub const fn with_archive(archive: &'a DamlArchive<'a>, filter_mode: RenderFilterMode) -> Self {
        Self {
            mode: RenderMode::Full(archive),
            filter_mode,
        }
    }

    pub const fn archive(&self) -> &DamlArchive<'_> {
        match &self.mode {
            RenderMode::Intermediate(archive) => archive,
            RenderMode::Full(archive) => archive,
        }
    }

    pub const fn filter_mode(&self) -> RenderFilterMode {
        self.filter_mode
    }
}

/// Rendering mode.
#[derive(Debug)]
pub enum RenderMode<'a> {
    Intermediate(DamlArchive<'a>),
    Full(&'a DamlArchive<'a>),
}

impl Default for RenderMode<'_> {
    fn default() -> Self {
        RenderMode::Intermediate(DamlArchive::default())
    }
}

/// Rendering filter mode.
#[derive(Debug, Clone, Copy)]
pub enum RenderFilterMode {
    /// Exclude only fields with type constructors that contain Higher Kinded Types (HKT) only.
    HigherKindedType,
    /// Exclude all non-serializable fields.
    NonSerializable,
}

impl Default for RenderFilterMode {
    fn default() -> Self {
        RenderFilterMode::HigherKindedType
    }
}
