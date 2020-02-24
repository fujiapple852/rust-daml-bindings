#[derive(Debug)]
pub struct DamlTypeVar<'a> {
    pub var: &'a str,
    pub kind: DamlKind,
}

impl<'a> DamlTypeVar<'a> {
    pub fn new(var: &'a str, kind: DamlKind) -> Self {
        Self {
            var,
            kind,
        }
    }
}

#[derive(Debug)]
pub enum DamlKind {
    Star,
    Arrow(Box<DamlArrow>),
    Nat,
}

#[derive(Debug)]
pub struct DamlArrow {
    pub params: Vec<DamlKind>,
    pub result: DamlKind,
}

impl DamlArrow {
    pub fn new(params: Vec<DamlKind>, result: DamlKind) -> Self {
        Self {
            params,
            result,
        }
    }
}
