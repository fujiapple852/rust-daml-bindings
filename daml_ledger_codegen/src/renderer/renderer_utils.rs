use crate::element::DamlType;
use heck::SnakeCase;
use itertools::Itertools;
use proc_macro2::{Ident, Span, TokenStream};
use quote::quote;
use std::convert::AsRef;

/// Quote a string as an identifier.
pub fn quote_ident(value: impl AsRef<str>) -> TokenStream {
    let ident = Ident::new(value.as_ref(), Span::call_site());
    quote!(#ident)
}

/// Escape and quote a string as an identifier.
pub fn quote_escaped_ident(value: impl AsRef<str>) -> TokenStream {
    quote_ident(escape_identifier(value))
}

/// Convert module path to a String.
pub fn to_module_path(path: &[&str]) -> String {
    path.iter().join(".")
}

/// Convert a string to a valid rust identifier.
pub fn to_rust_identifier(value: impl AsRef<str>) -> String {
    escape_identifier(value.as_ref().to_snake_case())
}

/// Determine if this type supported by the code generator.
pub fn is_supported_type(ty: &DamlType) -> bool {
    match ty {
        DamlType::Arrow | DamlType::Var | DamlType::Update | DamlType::Scenario | DamlType::Any | DamlType::TypeRep =>
            false,
        DamlType::List(inner) | DamlType::TextMap(inner) | DamlType::Optional(inner) => is_supported_type(&inner),
        _ => true,
    }
}

fn escape_identifier(value: impl AsRef<str>) -> String {
    let mut sanitized_ident = value.as_ref().replace("-", "_").replace("$", "_").replace(".", "_");
    escape_keyword(&mut sanitized_ident);
    sanitized_ident
}

fn escape_keyword(ident: &mut String) -> &mut String {
    match ident.as_str() {
        "as" | "break" | "const" | "continue" | "else" | "enum" | "false" | "fn" | "for" | "if" | "impl" | "in"
        | "let" | "loop" | "match" | "mod" | "move" | "mut" | "pub" | "ref" | "return" | "static" | "struct"
        | "trait" | "true" | "type" | "unsafe" | "use" | "where" | "while" | "dyn" | "abstract" | "become" | "box"
        | "do" | "final" | "macro" | "override" | "priv" | "typeof" | "unsized" | "virtual" | "yield" | "async"
        | "await" | "try" | "self" | "super" | "extern" | "crate" => *ident += "_",
        _ => (),
    }
    ident
}
