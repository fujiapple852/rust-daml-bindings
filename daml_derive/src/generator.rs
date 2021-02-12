use crate::convert::{
    data_type_string_from_type, extract_all_choices, extract_enum, extract_record, extract_template, extract_variant,
    AttrChoice, AttrRecord, AttrTemplate, AttrVariant,
};
use crate::CodeGeneratorParameters;
use daml_codegen::generator::{ModuleMatcher, RenderMethod};
use daml_codegen::renderer::full::{
    quote_choice, quote_daml_enum, quote_daml_record, quote_daml_template, quote_daml_variant,
};
use daml_codegen::renderer::quote_archive;
use daml_codegen::renderer::RenderContext;
use daml_lf::element::{DamlChoice, DamlEnum, DamlRecord, DamlTemplate, DamlVariant};
use daml_lf::DarFile;
use darling::FromMeta;
use quote::quote;
use syn::{AttributeArgs, Data, DataStruct, DeriveInput, Fields, ItemImpl};

/// Generate a Rust `TokenStream` representing the supplied DAML Archive.
pub fn generate_tokens(args: AttributeArgs) -> proc_macro::TokenStream {
    let params: CodeGeneratorParameters = CodeGeneratorParameters::from_list(&args).unwrap_or_else(|e| panic!("{}", e));
    let archive = DarFile::from_file(&params.dar_file)
        .unwrap_or_else(|e| panic!("failed to load Dar file from {}, error was: {}", &params.dar_file, e.to_string()));
    let filters: Vec<_> = params.module_filter_regex.iter().map(String::as_str).collect();
    let render_method = match &params.mode {
        Some(name) if name.to_ascii_lowercase() == "intermediate" => RenderMethod::Intermediate,
        Some(name) if name.to_ascii_lowercase() == "full" => RenderMethod::Full,
        Some(name) => panic!("unknown mode: {}, expected Intermediate or Full", name),
        _ => RenderMethod::Full,
    };
    let applied =
        archive.apply(|archive| ModuleMatcher::new(&filters).map(|mm| quote_archive(archive, &mm, &render_method)));
    match applied {
        Ok(Ok(tokens)) => proc_macro::TokenStream::from(tokens),
        Ok(Err(e)) => panic!("failed to generate DAML code: {0}", e),
        Err(e) => panic!("DAML-LF error in DAML code generator: {0}", e.to_string()),
    }
}

pub fn generate_template(input: DeriveInput, package_id: String, module_name: String) -> proc_macro::TokenStream {
    let struct_name = input.ident.to_string();
    match &input.data {
        Data::Struct(DataStruct {
            fields: Fields::Named(fields_named),
            ..
        }) => {
            let template: AttrTemplate = extract_template(struct_name, package_id, module_name, fields_named);
            let daml_template = DamlTemplate::from(&template);
            let ctx = RenderContext::default();
            let expanded = quote_daml_template(&ctx, &daml_template);
            proc_macro::TokenStream::from(expanded)
        },
        _ => panic!("the DamlTemplate attribute may only be applied to a named struct type"),
    }
}

pub fn generate_choices(input: ItemImpl) -> proc_macro::TokenStream {
    let struct_name = data_type_string_from_type(input.self_ty.as_ref());
    let all_choices: Vec<AttrChoice> = extract_all_choices(&input.items);
    let all_daml_choices: Vec<DamlChoice<'_>> = all_choices.iter().map(DamlChoice::from).collect();
    let ctx = RenderContext::default();
    let all_choice_methods_tokens = quote_choice(&ctx, &struct_name, &all_daml_choices);
    proc_macro::TokenStream::from(all_choice_methods_tokens)
}

pub fn generate_data_struct(input: DeriveInput) -> proc_macro::TokenStream {
    let struct_name = input.ident.to_string();
    let tokens = match input.data {
        Data::Struct(data_struct) => match &data_struct.fields {
            Fields::Named(fields_named) => {
                let record: AttrRecord = extract_record(struct_name, fields_named, &input.generics);
                let daml_record = DamlRecord::from(&record);
                let ctx = RenderContext::default();
                quote_daml_record(&ctx, &daml_record)
            },
            Fields::Unnamed(_) => panic!("tuple struct not supported"),
            Fields::Unit => panic!("unit struct not supported"),
        },
        _ => panic!("the DamlData attribute may only be applied to struct types"),
    };
    let expanded = quote!(
        #tokens
    );
    proc_macro::TokenStream::from(expanded)
}

pub fn generate_data_variant(input: DeriveInput) -> proc_macro::TokenStream {
    let variant_name = input.ident.to_string();
    let tokens = match input.data {
        Data::Enum(data_enum) => {
            let variant: AttrVariant = extract_variant(variant_name, &data_enum, &input.generics);
            let daml_variant = DamlVariant::from(&variant);
            let ctx = RenderContext::default();
            quote_daml_variant(&ctx, &daml_variant)
        },
        _ => panic!("the DamlVariant attribute may only be applied to enum types"),
    };
    let expanded = quote!(
        #tokens
    );
    proc_macro::TokenStream::from(expanded)
}

pub fn generate_data_enum(input: DeriveInput) -> proc_macro::TokenStream {
    let enum_name = input.ident.to_string();
    let tokens = match input.data {
        Data::Enum(data_enum) => {
            let enum_variants = extract_enum(enum_name, &data_enum, &input.generics);
            let daml_enum = DamlEnum::from(&enum_variants);
            let ctx = RenderContext::default();
            quote_daml_enum(&ctx, &daml_enum)
        },
        _ => panic!("the DamlEnum attribute may only be applied to enum types"),
    };
    let expanded = quote!(
        #tokens
    );
    proc_macro::TokenStream::from(expanded)
}
