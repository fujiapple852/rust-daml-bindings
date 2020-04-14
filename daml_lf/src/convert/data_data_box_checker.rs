use crate::convert::data_payload::{DamlDataEnrichedPayload, DamlDataWrapper};
use crate::convert::field_payload::DamlFieldWrapper;
use crate::convert::interned::InternableDottedName;
use crate::convert::resolver::resolve_data_ref;
use crate::convert::type_payload::{DamlTypePayload, DamlTypeWrapper};
use crate::error::DamlLfConvertResult;
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
    pub fn should_box(parent_data: DamlDataWrapper<'a>, child_data: DamlDataWrapper<'a>) -> DamlLfConvertResult<bool> {
        Self::new(parent_data).check_data(child_data)
    }

    fn new(parent_data: DamlDataWrapper<'a>) -> Self {
        DamlDataBoxChecker {
            search_data: parent_data,
            visited: HashSet::new(),
        }
    }

    fn check_data(&mut self, data: DamlDataWrapper<'a>) -> DamlLfConvertResult<bool> {
        match data.payload {
            DamlDataEnrichedPayload::Record(record) =>
                self.check_field_types(record.fields.iter().map(|field| data.wrap(field))),
            DamlDataEnrichedPayload::Variant(variant) =>
                self.check_field_types(variant.fields.iter().map(|field| data.wrap(field))),
            _ => Ok(false),
        }
    }

    fn check_field_types(&mut self, fields: impl Iterator<Item = DamlFieldWrapper<'a>>) -> DamlLfConvertResult<bool> {
        self.check_types(fields.map(|field| field.wrap(&field.payload.ty)))
    }

    fn check_types(&mut self, types: impl Iterator<Item = DamlTypeWrapper<'a>>) -> DamlLfConvertResult<bool> {
        let types_iter = types.map(|ty| self.check_type(ty));
        Ok(itertools::process_results(types_iter, |mut iter| iter.any(|found| found))?)
    }

    fn check_type(&mut self, daml_type: DamlTypeWrapper<'a>) -> DamlLfConvertResult<bool> {
        Ok(match daml_type.payload {
            DamlTypePayload::Optional(nested) => self.check_type(daml_type.wrap(nested.as_ref()))?,
            DamlTypePayload::DataRef(data_ref) => {
                let data = resolve_data_ref(daml_type.wrap(data_ref))?;
                data.payload == self.search_data.payload
                    || self.memoized_visit_data(data)?
                    || self.check_types(data_ref.type_arguments.iter().map(|ty| daml_type.wrap(ty)))?
            },
            DamlTypePayload::ContractId(_)
            | DamlTypePayload::Int64
            | DamlTypePayload::Numeric
            | DamlTypePayload::Text
            | DamlTypePayload::Timestamp
            | DamlTypePayload::Party
            | DamlTypePayload::Bool
            | DamlTypePayload::Unit
            | DamlTypePayload::Date
            | DamlTypePayload::List(_)
            | DamlTypePayload::Update
            | DamlTypePayload::Scenario
            | DamlTypePayload::TextMap(_)
            | DamlTypePayload::Var(_)
            | DamlTypePayload::Arrow
            | DamlTypePayload::Any
            | DamlTypePayload::TypeRep => false,
        })
    }

    fn memoized_visit_data(&mut self, data: DamlDataWrapper<'a>) -> DamlLfConvertResult<bool> {
        if self.visited.contains(&data.payload.name()) {
            Ok(false)
        } else {
            self.visited.insert(data.payload.name());
            self.check_data(data)
        }
    }
}
