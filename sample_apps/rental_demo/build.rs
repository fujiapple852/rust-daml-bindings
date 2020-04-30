use daml::codegen::generator::{daml_codegen, ModuleOutputMode, RenderMethod};

const DAR_PATH: &str = "resources/rental/archive/rental-0_0_1-sdk_1_0_1-lf_1_8.dar";
const OUTPUT_PATH: &str = "src/autogen";

fn main() {
    daml_codegen(DAR_PATH, OUTPUT_PATH, &[], RenderMethod::Full, ModuleOutputMode::Combined)
        .expect("failed to generate code for DAML archive");
}
