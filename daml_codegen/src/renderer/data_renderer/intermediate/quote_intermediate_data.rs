use crate::renderer::field_renderer::quote_fields;
use crate::renderer::type_renderer::quote_type;
use crate::renderer::{quote_escaped_ident, to_module_path, IsRenderable, RenderContext};
use daml_lf::element::{DamlChoice, DamlEnum, DamlField, DamlRecord, DamlTemplate, DamlType, DamlVariant};
use heck::SnakeCase;
use proc_macro2::TokenStream;
use quote::quote;

pub fn quote_daml_template(ctx: &RenderContext<'_>, template: &DamlTemplate<'_>) -> TokenStream {
    let package_id = template.package_id();
    let module_name = to_module_path(template.module_path());
    let name_tokens = quote_escaped_ident(template.name());
    let supported_fields: Vec<_> =
        template.fields().iter().filter(|&field| IsRenderable::new(ctx).check_type(field.ty())).collect();
    let all_fields_tokens = quote_fields(supported_fields.as_slice());
    let all_choices_tokens: Vec<_> = template.choices().iter().map(|c| quote_choice(ctx, c)).collect();
    quote!(
        #[DamlTemplate(package_id = #package_id, module_name = #module_name)]
        pub struct #name_tokens {
            #all_fields_tokens
        }
        #[DamlChoices]
        impl #name_tokens {
            #( #all_choices_tokens )*
        }
    )
}

fn quote_choice(ctx: &RenderContext<'_>, choice: &DamlChoice<'_>) -> TokenStream {
    let choice_name_tokens = quote_escaped_ident(choice.name());
    let function_name_tokens = quote_escaped_ident(choice.name().to_snake_case());
    let supported_fields: Vec<_> =
        choice.fields().iter().filter(|&field| IsRenderable::new(ctx).check_type(field.ty())).collect();
    let arg_tokens = quote_fields(supported_fields.as_slice());
    quote!(
        #[#choice_name_tokens]
        pub fn #function_name_tokens(&self, #arg_tokens) {}
    )
}

pub fn quote_daml_record(ctx: &RenderContext<'_>, record: &DamlRecord<'_>) -> TokenStream {
    let name_tokens = quote_escaped_ident(&record.name());
    let supported_fields: Vec<_> =
        record.fields().iter().filter(|&field| IsRenderable::new(ctx).check_type(field.ty())).collect();
    let all_fields_tokens = quote_fields(supported_fields.as_slice());
    quote!(
        #[DamlData]
        pub struct #name_tokens {
            #all_fields_tokens
        }
    )
}

pub fn quote_daml_variant(ctx: &RenderContext<'_>, variant: &DamlVariant<'_>) -> TokenStream {
    let name_tokens = quote_escaped_ident(variant.name());
    let all_variants_tokens: Vec<_> = variant
        .fields()
        .iter()
        .filter_map(|field| {
            if IsRenderable::new(ctx).check_type(field.ty()) {
                Some(quote_variant_field(field))
            } else {
                None
            }
        })
        .collect();
    quote!(
        #[DamlVariant]
        pub enum #name_tokens {
            #( #all_variants_tokens ),*
        }
    )
}

fn quote_variant_field(field: &DamlField<'_>) -> TokenStream {
    let name_tokens = quote_escaped_ident(field.name());
    if let DamlType::Unit = field.ty() {
        quote!(
            #name_tokens
        )
    } else {
        let type_tokens = quote_type(field.ty());
        quote!(
            #name_tokens (#type_tokens)
        )
    }
}

pub fn quote_daml_enum(_ctx: &RenderContext<'_>, data_enum: &DamlEnum<'_>) -> TokenStream {
    let name_tokens = quote_escaped_ident(data_enum.name());
    let all_enum_constructors: Vec<_> = data_enum.constructors().iter().map(|field| quote_enum_field(field)).collect();
    quote!(
        #[DamlEnum]
        pub enum #name_tokens {
            #( #all_enum_constructors ),*
        }
    )
}

fn quote_enum_field(field: &str) -> TokenStream {
    let name = quote_escaped_ident(field);
    quote!(
        #name
    )
}
