use crate::element::daml_type::DamlType;

#[derive(Debug)]
pub struct DamlField<'a> {
    pub name: &'a str,
    pub ty: DamlType<'a>,
}

impl<'a> DamlField<'a> {
    pub fn new(name: &'a str, ty: DamlType<'a>) -> Self {
        Self {
            name,
            ty,
        }
    }
}
