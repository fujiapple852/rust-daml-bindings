use crate::quote::quote_ident;
use proc_macro2::TokenStream;
use quote::quote;

/// Generate the `FooContract` struct name.
pub fn quote_contract_struct_name(struct_name: &str) -> TokenStream {
    quote_ident(format!("{}Contract", struct_name))
}

/// Generate the `FooContract` struct and methods.
pub fn quote_contract_struct_and_impl(struct_name: &str) -> TokenStream {
    let struct_tokens = quote_contract_struct(struct_name);
    let impl_getters_tokens = quote_contract_struct_impl_getters(struct_name);
    let impl_try_from_tokens = quote_contract_struct_impl_try_from(struct_name);
    quote!(
        #struct_tokens
        #impl_getters_tokens
        #impl_try_from_tokens
    )
}

/// Generate `struct FooContract {...}` struct.
fn quote_contract_struct(struct_name: &str) -> TokenStream {
    let struct_name_tokens = quote_ident(struct_name);
    let contract_struct_name_tokens = quote_contract_struct_name(struct_name);
    quote!(
        #[derive(Debug)]
        pub struct #contract_struct_name_tokens {
            id: String,
            data: #struct_name_tokens,
        }
    )
}

/// Generate `FooContract::id()` & `FooContract::data()` methods.
fn quote_contract_struct_impl_getters(struct_name: &str) -> TokenStream {
    let struct_name_tokens = quote_ident(struct_name);
    let contract_struct_name_tokens = quote_contract_struct_name(struct_name);
    quote!(
        impl #contract_struct_name_tokens {
            pub fn id(&self) -> &str {
                &self.id
            }
            pub fn data(&self) -> &#struct_name_tokens {
                &self.data
            }
        }
    )
}

/// Generate the `TryFrom<DamlCreatedEvent> for FooContract` method.
fn quote_contract_struct_impl_try_from(struct_name: &str) -> TokenStream {
    let contract_struct_name_tokens = quote_contract_struct_name(struct_name);
    quote!(
        impl TryFrom<DamlCreatedEvent> for #contract_struct_name_tokens {
            type Error = DamlError;

            fn try_from(event: DamlCreatedEvent) -> Result<Self, Self::Error> {
                let contract_id = event.contract_id().to_owned();
                let record: DamlRecord = event.take_create_arguments();
                Ok(Self {
                    id: contract_id,
                    data: DamlValue::new_record(record).try_into()?,
                })
            }
        }
    )
}
