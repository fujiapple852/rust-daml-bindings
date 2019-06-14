use crate::daml_element::{data_type_string_from_path, DamlField, DamlType};
use syn::{ArgCaptured, Attribute, FnArg, ImplItem, ImplItemMethod, Pat, Type};

#[derive(Debug)]
pub struct DamlChoice {
    pub choice_name: String,
    pub choice_method_name: String,
    pub choice_arguments: Vec<DamlField>,
}

pub fn get_single_attr_method(impl_item: &ImplItem) -> Option<(DamlChoice)> {
    if let ImplItem::Method(ImplItemMethod {
        attrs,
        sig,
        ..
    }) = impl_item
    {
        match attrs.as_slice() {
            [Attribute {
                path,
                ..
            }] => Some(DamlChoice {
                choice_name: data_type_string_from_path(path),
                choice_method_name: sig.ident.to_string(),
                choice_arguments: sig.decl.inputs.iter().filter_map(self::simple_method_name_and_type).collect(),
            }),
            _ => None,
        }
    } else {
        None
    }
}

fn simple_method_name_and_type(arg: &FnArg) -> Option<DamlField> {
    match arg {
        FnArg::Captured(ArgCaptured {
            pat,
            ty,
            ..
        }) => match pat {
            Pat::Ident(pat_ident) => match ty {
                Type::Path(type_path) => Some(DamlField {
                    field_label: pat_ident.ident.clone().to_string(),
                    field_type: DamlType::from_path(&type_path.path),
                }),
                _ => None,
            },
            _ => None,
        },
        _ => None,
    }
}
