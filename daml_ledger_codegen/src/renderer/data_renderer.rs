use proc_macro2::TokenStream;

use self::full::quote_daml_enum as quote_daml_enum_full;
use self::full::quote_daml_record as quote_daml_record_full;
use self::full::quote_daml_template as quote_daml_template_full;
use self::full::quote_daml_variant as quote_daml_variant_full;
use self::intermediate::quote_daml_enum as quote_daml_enum_intermediate;
use self::intermediate::quote_daml_record as quote_daml_record_intermediate;
use self::intermediate::quote_daml_template as quote_daml_template_intermediate;
use self::intermediate::quote_daml_variant as quote_daml_variant_intermediate;
use crate::element::DamlData;
use crate::generator::RenderMethod;

use quote::quote;

pub mod full {
    mod quote_choices;
    mod quote_contract_struct;
    mod quote_enum;
    mod quote_main_struct;
    mod quote_template;
    mod quote_variant;

    pub use quote_choices::*;
    pub use quote_enum::*;
    pub use quote_main_struct::*;
    pub use quote_template::*;
    pub use quote_variant::*;
}

mod intermediate {
    mod quote_intermediate_data;

    pub use quote_intermediate_data::*;
}

pub fn quote_all_data(all_data: &[&DamlData], render_method: &RenderMethod) -> TokenStream {
    let all_data_tokens: Vec<_> = all_data.iter().map(|&dt| quote_data(dt, render_method)).collect();
    quote!(
        #( #all_data_tokens )*
    )
}

pub fn quote_data(data_type: &DamlData, render_method: &RenderMethod) -> TokenStream {
    match render_method {
        RenderMethod::Full => match data_type {
            DamlData::Template(template) => quote_daml_template_full(template),
            DamlData::Record(record) => quote_daml_record_full(record),
            DamlData::Variant(variant) => quote_daml_variant_full(variant),
            DamlData::Enum(data_enum) => quote_daml_enum_full(data_enum),
        },
        RenderMethod::Intermediate => match data_type {
            DamlData::Template(template) => quote_daml_template_intermediate(template),
            DamlData::Record(record) => quote_daml_record_intermediate(record),
            DamlData::Variant(variant) => quote_daml_variant_intermediate(variant),
            DamlData::Enum(data_enum) => quote_daml_enum_intermediate(data_enum),
        },
    }
}
