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

#[DamlVariant]
pub enum MyList {
    Val(String),
    Cons(ListItem),
}

#[DamlData]
pub struct ListItem {
    value: String,
    cons: Box<MyList>,
}

#[DamlData]
pub struct GenericStruct<T> {
    value: T,
    cons: DamlOptional<Box<GenericStruct<T>>>,
}

#[DamlVariant]
pub enum GenericVariant<T> {
    Base(T),
    Cons(Box<GenericVariant<T>>),
}

#[DamlData]
pub struct GenericData<T> {
    value: T,
}

#[DamlData]
pub struct ConcreteData {
    data_1: GenericData<DamlText>,
    data_2: DamlOptional<GenericData<Box<ConcreteData>>>,
}
