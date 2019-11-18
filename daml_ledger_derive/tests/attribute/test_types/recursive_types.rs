use daml::prelude::*;

#[DamlData]
pub struct InnerType {
    name: DamlText,
    boxed_outer: DamlOptional<Box<BoxedOuterType>>,
    outer: DamlOptional<OuterType>,
}

#[DamlData]
pub struct BoxedOuterType {
    name: DamlText,
    inner: InnerType,
}

#[DamlData]
pub struct OuterType {
    name: DamlText,
    inner: Box<InnerType>,
}
