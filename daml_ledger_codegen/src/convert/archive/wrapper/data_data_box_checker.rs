use crate::convert::archive::wrapper::payload::{DamlTypePayload, InternableDottedName};
use crate::convert::archive::wrapper::{DamlDataWrapper, DamlFieldWrapper, DamlTypeWrapper};
use std::collections::HashSet;

/// Utility for determining whether a struct field should be boxed.
pub struct DamlDataBoxChecker<'a> {
    search_data: DamlDataWrapper<'a>,
    visited: HashSet<InternableDottedName<'a>>,
}

impl<'a> DamlDataBoxChecker<'a> {
    /// Check if `parent_data` is referenced by `child_data` such that the reference to `child_data` should be `Box`'ed.
    ///
    /// Recursively check all fields of `child_data`, including all nested data types and type arguments, for
    /// references to `parent_data`.
    ///
    /// Note that type arguments to `DamlTypePayload::List` & `DamlTypePayload::TextMap` are not searched as these
    /// container types are boxed already.
    pub fn should_box(parent_data: DamlDataWrapper<'a>, child_data: DamlDataWrapper<'a>) -> bool {
        Self::new(parent_data).check_data(child_data)
    }

    fn new(parent_data: DamlDataWrapper<'a>) -> Self {
        DamlDataBoxChecker {
            search_data: parent_data,
            visited: HashSet::new(),
        }
    }

    fn check_data(&mut self, data: DamlDataWrapper<'a>) -> bool {
        match data {
            DamlDataWrapper::Record(record) => self.check_field_types(record.fields()),
            DamlDataWrapper::Variant(variant) => self.check_field_types(variant.fields()),
            _ => false,
        }
    }

    fn check_field_types(&mut self, fields: impl Iterator<Item = DamlFieldWrapper<'a>>) -> bool {
        self.check_types(fields.map(DamlFieldWrapper::ty))
    }

    fn check_types(&mut self, mut type_arguments: impl Iterator<Item = DamlTypeWrapper<'a>>) -> bool {
        type_arguments.any(|ty| self.check_type(ty))
    }

    fn check_type(&mut self, daml_type: DamlTypeWrapper<'a>) -> bool {
        match daml_type.payload {
            DamlTypePayload::Optional(_) => self.check_type(daml_type.nested_type()),
            DamlTypePayload::DataRef(_) => {
                let data_ref = daml_type.data_ref();
                let data = data_ref.get_data();
                data == self.search_data
                    || self.memoized_visit_data(data)
                    || self.check_types(data_ref.type_arguments())
            },
            _ => false,
        }
    }

    fn memoized_visit_data(&mut self, data: DamlDataWrapper<'a>) -> bool {
        if self.visited.contains(&data.name()) {
            false
        } else {
            self.visited.insert(data.name());
            self.check_data(data)
        }
    }
}
