use daml_lf::element::DamlArchive;

/// Contextual information required during code generation.
#[derive(Debug)]
pub enum RenderContext<'a> {
    Intermediate(DamlArchive<'a>),
    Full(&'a DamlArchive<'a>),
}

impl Default for RenderContext<'_> {
    fn default() -> Self {
        RenderContext::Intermediate(DamlArchive::default())
    }
}

impl<'a> RenderContext<'a> {
    /// Create a new `RenderContext` with a `DamlArchive`.
    pub fn with_archive(archive: &'a DamlArchive<'a>) -> Self {
        RenderContext::Full(archive)
    }

    pub fn archive(&self) -> &DamlArchive<'_> {
        match self {
            RenderContext::Intermediate(archive) => archive,
            RenderContext::Full(archive) => archive,
        }
    }
}
