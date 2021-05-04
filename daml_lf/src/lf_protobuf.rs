#![allow(clippy::all, clippy::pedantic)]
pub mod com {
    pub mod digitalasset {
        pub mod daml_lf_1 {
            include!(concat!(env!("OUT_DIR"), "/daml_lf_1.rs"));
        }
        pub mod daml_lf {
            include!(concat!(env!("OUT_DIR"), "/daml_lf_1_13.rs"));
        }
    }
}
