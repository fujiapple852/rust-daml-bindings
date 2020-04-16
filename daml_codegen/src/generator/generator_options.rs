/// Render full or intermediate annotated Rust types.
#[derive(Debug)]
pub enum RenderMethod {
    Full,
    Intermediate,
}

/// Render each module as a separate file or combined in a single file.
#[derive(Debug)]
pub enum ModuleOutputMode {
    Combined,
    Separate,
}
