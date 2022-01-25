use daml_codegen::generator::{daml_codegen_internal, ModuleOutputMode, RenderMethod};

use clap::{App, Arg};

fn main() {
    let matches = App::new("Daml codegen for Rust")
        .version("0.1.0")
        .about("Generate Rust code for working with Daml types")
        .arg(Arg::new("dar").help("Sets the input Dar file to use").required(true).index(1))
        .arg(Arg::new("output").short('o').long("output-dir").takes_value(true).help("Sets the output path"))
        .arg(
            Arg::new("verbose")
                .short('v')
                .long("verbose")
                .multiple_occurrences(true)
                .help("Sets the level of verbosity"),
        )
        .arg(
            Arg::new("filter")
                .short('f')
                .long("module-filter")
                .takes_value(true)
                .multiple_values(true)
                .help("Sets the regex module filter to apply"),
        )
        .arg(Arg::new("intermediate").short('i').long("render-intermediate").help("Generate intermediate types"))
        .arg(Arg::new("combine").short('c').long("combine-modules").help("Combine modules as a single file"))
        .get_matches();
    let dar_file = matches.value_of("dar").unwrap();
    let output_path = matches.value_of("output").unwrap_or(".");
    let filters: Vec<_> = if matches.is_present("filter") {
        matches.values_of("filter").unwrap().collect()
    } else {
        vec![]
    };
    let render_method = if matches.is_present("intermediate") {
        RenderMethod::Intermediate
    } else {
        RenderMethod::Full
    };
    let module_output_mode = if matches.is_present("combine") {
        ModuleOutputMode::Combined
    } else {
        ModuleOutputMode::Separate
    };
    daml_codegen_internal(dar_file, output_path, &filters, render_method, module_output_mode)
        .expect("failed to generate code for Daml archive");
}
