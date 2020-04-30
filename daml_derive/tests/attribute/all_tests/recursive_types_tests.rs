use crate::attribute::test_types::recursive_types::{
    BoxedOuterType, ConcreteData, GenericData, GenericVariant, InnerType, ListItem, MyList, OuterType,
};
use crate::common::test_utils::TestResult;
use daml::api::data::value::DamlValue;
use daml::prelude::{DamlDeserializeInto, DamlSerializeInto};

#[test]
pub fn test_mutual_recursive_type() -> TestResult {
    let inner = InnerType::new(
        "The Inner",
        Some(Box::new(BoxedOuterType::new("The Boxed Outer", InnerType::new("The Inner 2", None, None)))),
        OuterType::new("The Outer", InnerType::new("The Inner 3", None, None)),
    );
    let value: DamlValue = inner.clone().serialize_into();
    let inner_again = value.deserialize_into()?;
    assert_eq!(inner, inner_again);
    Ok(())
}

#[test]
pub fn test_recursive_variant() -> TestResult {
    let my_list = MyList::Cons(ListItem::new("item 1", MyList::Val("item 2".to_owned())));
    let value: DamlValue = my_list.clone().serialize_into();
    let my_list_again = value.deserialize_into()?;
    assert_eq!(my_list, my_list_again);
    Ok(())
}

#[test]
pub fn test_recursive_generic_struct() -> TestResult {
    let concrete = GenericVariant::Cons(Box::new(GenericVariant::Base(100)));
    let value: DamlValue = concrete.clone().serialize_into();
    let concrete_again = value.deserialize_into()?;
    assert_eq!(concrete, concrete_again);
    Ok(())
}

#[test]
pub fn test_recursive_generic_variant() -> TestResult {
    let concrete = ConcreteData::new(
        GenericData::new("text 2".to_owned()),
        Some(GenericData::new(ConcreteData::new(GenericData::new("text 1".to_owned()), None))),
    );
    let value: DamlValue = concrete.clone().serialize_into();
    let concrete_again = value.deserialize_into()?;
    assert_eq!(concrete, concrete_again);
    Ok(())
}
