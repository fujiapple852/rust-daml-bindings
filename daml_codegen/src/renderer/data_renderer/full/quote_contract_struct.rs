use proc_macro2::TokenStream;

use crate::renderer::quote_escaped_ident;
use quote::quote;

/// Generate the `FooContract` struct name.
pub fn quote_contract_struct_name(struct_name: &str) -> TokenStream {
    quote_escaped_ident(format!("{}Contract", struct_name))
}

/// Generate the `FooContractId` struct name.
pub fn quote_contract_id_struct_name(struct_name: &str) -> TokenStream {
    quote_escaped_ident(format!("{}ContractId", struct_name))
}

/// Generate the `FooContract` struct and methods.
pub fn quote_contract_struct_and_impl(struct_name: &str) -> TokenStream {
    let contract_struct_tokens = quote_contract_struct(struct_name);
    let contract_impl_getters_tokens = quote_contract_struct_impl_getters(struct_name);
    let contract_impl_try_from_tokens = quote_contract_struct_impl_try_from(struct_name);
    let contract_id_struct_tokens = quote_contract_id_struct(struct_name);
    let contract_id_impl_getters_tokens = quote_contract_id_struct_impl_getters(struct_name);
    let contract_id_impl_try_from_tokens = quote_contract_id_struct_impl_try_from(struct_name);
    quote!(
        #contract_struct_tokens
        #contract_impl_getters_tokens
        #contract_impl_try_from_tokens
        #contract_id_struct_tokens
        #contract_id_impl_getters_tokens
        #contract_id_impl_try_from_tokens
    )
}

/// Generate `struct FooContract {...}` struct.
fn quote_contract_struct(struct_name: &str) -> TokenStream {
    let struct_name_tokens = quote_escaped_ident(struct_name);
    let contract_struct_name_tokens = quote_contract_struct_name(struct_name);
    let contract_id_struct_name_tokens = quote_contract_id_struct_name(struct_name);
    quote!(
        #[derive(Debug)]
        pub struct #contract_struct_name_tokens {
            pub id: #contract_id_struct_name_tokens,
            pub data: #struct_name_tokens,
        }
    )
}

/// Generate `FooContract::id()` & `FooContract::data()` methods.
fn quote_contract_struct_impl_getters(struct_name: &str) -> TokenStream {
    let struct_name_tokens = quote_escaped_ident(struct_name);
    let contract_struct_name_tokens = quote_contract_struct_name(struct_name);
    let contract_id_struct_name_tokens = quote_contract_id_struct_name(struct_name);
    quote!(
        impl #contract_struct_name_tokens {
            pub fn id(&self) -> &#contract_id_struct_name_tokens {
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
    let contract_id_struct_name_tokens = quote_contract_id_struct_name(struct_name);
    quote!(
        impl std::convert::TryFrom<DamlCreatedEvent> for #contract_struct_name_tokens {

            type Error = DamlError;
            fn try_from(event: DamlCreatedEvent) -> std::result::Result<Self, <#contract_struct_name_tokens as std::convert::TryFrom<DamlCreatedEvent>>::Error> {
                let contract_id = event.contract_id().to_owned();
                let record: DamlRecord = event.take_create_arguments();
                Ok(Self {
                    id: #contract_id_struct_name_tokens::try_from(DamlContractId::new(contract_id))?,
                    data: DamlValue::new_record(record).deserialize_into()?,
                })
            }
        }
    )
}

/// Generate `struct FooContractId {...}` struct.
fn quote_contract_id_struct(struct_name: &str) -> TokenStream {
    let contract_id_struct_name_tokens = quote_contract_id_struct_name(struct_name);
    quote!(
        #[derive(Debug)]
        pub struct #contract_id_struct_name_tokens {
            pub contract_id: DamlContractId,
        }
    )
}

/// Generate `struct FooContractId::try_from()` struct.
fn quote_contract_id_struct_impl_try_from(struct_name: &str) -> TokenStream {
    let contract_id_struct_name_tokens = quote_contract_id_struct_name(struct_name);
    quote!(
        impl std::convert::TryFrom<DamlContractId> for #contract_id_struct_name_tokens {
            type Error = DamlError;
            fn try_from(contract_id: DamlContractId) -> std::result::Result<Self, <#contract_id_struct_name_tokens as std::convert::TryFrom<DamlContractId>>::Error> {
                Ok(Self {
                    contract_id,
                })
            }
        }
    )
}

/// Generate `FooContractId::contract_id()` methods.
fn quote_contract_id_struct_impl_getters(struct_name: &str) -> TokenStream {
    let contract_id_struct_name_tokens = quote_contract_id_struct_name(struct_name);
    quote!(
        impl #contract_id_struct_name_tokens {
            pub fn contract_id(&self) -> &DamlContractId {
                &self.contract_id
            }
        }
    )
}
