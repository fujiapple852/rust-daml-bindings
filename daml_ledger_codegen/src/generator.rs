mod code_generator;
mod generator_options;
mod module_matcher;

mod archive_code_generator {
    pub mod combined;
    pub mod separate;
}

pub use code_generator::*;
pub use generator_options::*;

#[doc(hidden)]
pub use module_matcher::*;

#[doc(hidden)]
pub mod attribute_code_generator {
    mod choices;
    mod data;
    mod template;

    pub use choices::*;
    pub use data::*;
    pub use template::*;
}
