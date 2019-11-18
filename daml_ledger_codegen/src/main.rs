use daml_ledger_codegen::generator::{daml_codegen_internal, ModuleOutputMode, RenderMethod};

use clap::{App, Arg};

fn main() {
    let matches = App::new("DAML codegen for Rust")
        .version("0.1.0")
        .about("Generate Rust code for working with DAML types")
        .arg(Arg::with_name("dar").help("Sets the input Dar file to use").required(true).index(1))
        .arg(Arg::with_name("output").short("o").long("output-dir").takes_value(true).help("Sets the output path"))
        .arg(Arg::with_name("verbose").short("v").long("verbose").multiple(true).help("Sets the level of verbosity"))
        .arg(
            Arg::with_name("filter")
                .short("f")
                .long("module-filter")
                .takes_value(true)
                .multiple(true)
                .help("Sets the regex module filter to apply"),
        )
        .arg(Arg::with_name("intermediate").short("i").long("render-intermediate").help("Generate intermediate types"))
        .arg(Arg::with_name("combine").short("c").long("combine-modules").help("Combine modules as a single file"))
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
        .expect("failed to generate code for DAML archive");
}
