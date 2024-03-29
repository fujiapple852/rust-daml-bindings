use proc_macro2::TokenStream;

use self::full::quote_daml_enum as quote_daml_enum_full;
use self::full::quote_daml_record as quote_daml_record_full;
use self::full::quote_daml_template as quote_daml_template_full;
use self::full::quote_daml_variant as quote_daml_variant_full;
use self::intermediate::quote_daml_enum as quote_daml_enum_intermediate;
use self::intermediate::quote_daml_record as quote_daml_record_intermediate;
use self::intermediate::quote_daml_template as quote_daml_template_intermediate;
use self::intermediate::quote_daml_variant as quote_daml_variant_intermediate;
use crate::generator::RenderMethod;
use daml_lf::element::DamlData;

use crate::renderer::RenderContext;
use quote::quote;

pub mod full {
    mod quote_choices;
    mod quote_contract_struct;
    mod quote_enum;
    mod quote_generic_common;
    mod quote_main_struct;
    mod quote_method_params;
    mod quote_template;
    mod quote_variant;

    pub use quote_choices::*;
    pub use quote_enum::*;
    pub use quote_generic_common::*;
    pub use quote_main_struct::*;
    pub use quote_method_params::*;
    pub use quote_template::*;
    pub use quote_variant::*;
}

mod intermediate {
    mod quote_intermediate_data;

    pub use quote_intermediate_data::*;
}

// TODO make this take an iterator
pub fn quote_all_data(
    ctx: &RenderContext<'_>,
    all_data: &[&DamlData<'_>],
    render_method: &RenderMethod,
) -> TokenStream {
    let all_data_tokens: Vec<_> = all_data.iter().map(|&dt| quote_data(ctx, dt, render_method)).collect();
    quote!(
        #( #all_data_tokens )*
    )
}

pub fn quote_data(ctx: &RenderContext<'_>, data_type: &DamlData<'_>, render_method: &RenderMethod) -> TokenStream {
    match render_method {
        RenderMethod::Full => match data_type {
            DamlData::Template(template) => quote_daml_template_full(ctx, template),
            DamlData::Record(record) => quote_daml_record_full(ctx, record),
            DamlData::Variant(variant) => quote_daml_variant_full(ctx, variant),
            DamlData::Enum(data_enum) => quote_daml_enum_full(ctx, data_enum),
        },
        RenderMethod::Intermediate => match data_type {
            DamlData::Template(template) => quote_daml_template_intermediate(ctx, template),
            DamlData::Record(record) => quote_daml_record_intermediate(ctx, record),
            DamlData::Variant(variant) => quote_daml_variant_intermediate(ctx, variant),
            DamlData::Enum(data_enum) => quote_daml_enum_intermediate(ctx, data_enum),
        },
    }
}
