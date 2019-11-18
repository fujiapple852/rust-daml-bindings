use crate::convert::archive::wrapper::payload::DamlTypePayload;
use crate::convert::archive::wrapper::{DamlDataWrapper, DamlFieldWrapper, DamlTypeWrapper};

pub struct DamlDataFinder<'a> {
    search_data: DamlDataWrapper<'a>,
    visited: Vec<DamlDataWrapper<'a>>,
}

impl<'a> DamlDataFinder<'a> {
    pub fn new(search_data: DamlDataWrapper<'a>) -> Self {
        DamlDataFinder {
            search_data,
            visited: Default::default(),
        }
    }

    pub fn find(&mut self, data: DamlDataWrapper<'a>) -> bool {
        match data {
            DamlDataWrapper::Record(record) =>
                self.find_fields(record.fields().map(|field| field).collect::<Vec<_>>().as_slice()),
            DamlDataWrapper::Variant(variant) =>
                self.find_fields(variant.fields().map(|field| field).collect::<Vec<_>>().as_slice()),
            _ => false,
        }
    }

    fn find_fields(&mut self, fields: &[DamlFieldWrapper<'a>]) -> bool {
        fields.iter().any(|field| self.find_type(field.ty()))
    }

    fn find_type(&mut self, daml_type: DamlTypeWrapper<'a>) -> bool {
        match daml_type.payload {
            DamlTypePayload::Optional(_) => self.find_type(daml_type.nested_type()),
            DamlTypePayload::DataRef(_) => {
                let data = daml_type.data_ref().get_data();
                if self.is_visited(&data) {
                    false
                } else {
                    self.visited.push(data);
                    data == self.search_data || self.find(data)
                }
            },
            _ => false,
        }
    }

    fn is_visited(&self, data: &DamlDataWrapper<'a>) -> bool {
        self.visited.iter().any(|f| f == data)
    }
}
