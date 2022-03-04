/// Render full or intermediate annotated Rust types.
#[derive(Debug)]
pub enum RenderMethod {
    /// Render Rust types fully decomposed.
    Full,
    /// Render Rust types annotated with attributes such as `[DamlData]`.
    Intermediate,
}

/// Render each module as a separate file or combined in a single file.
#[derive(Debug)]
pub enum ModuleOutputMode {
    /// Render all Daml modules in a single Rust src file.
    Combined,
    /// Render each Daml module in a separate Rust src files.
    Separate,
}
