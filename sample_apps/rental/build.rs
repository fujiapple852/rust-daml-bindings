use daml_ledger_codegen::generator::{daml_codegen, ModuleOutputMode, RenderMethod};

const DAR_PATH: &str = "resources/rental/archive/rental-0.0.1-sdk_0.13.29.dar";
const OUTPUT_PATH: &str = "src/autogen";

fn main() {
    daml_codegen(DAR_PATH, OUTPUT_PATH, &[], RenderMethod::Full, ModuleOutputMode::Combined)
        .expect("failed to generate code for DAML archive");
}
