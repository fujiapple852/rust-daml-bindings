use daml::prelude::*;

/// The following is an example of how this Daml variant data type can be represented:
///
/// ```daml
/// data RecordArgument = RecordArgument with field_aaa: Int; field_bbb: [Text]
///
/// data AllVariantTypes =
///          NoArgument |
///          TupleStructPrimitive Text |
///          TupleStructListOfPrimitive [Int] |
///          TupleStructListOfRecord [RecordArgument] |
///          TupleStructMapOfPrimitive Map Party |
///          TupleStructMapOfRecord Map RecordArgument |
///          TupleStructOptionalOfPrimitive (Optional Bool) |
///          TupleStructOptionalOfRecord (Optional RecordArgument) |
///          TupleStructComplexType (Optional ([Int])) |
///          TupleStructRecord RecordArgument |
///          Record with field_aaa: Int; field_bbb: [Text]
/// ```
#[DamlVariant]
pub enum AllVariantTypes {
    NoArgument,
    TupleStructPrimitive(DamlText),
    TupleStructListOfPrimitive(DamlList<DamlInt64>),
    TupleStructListOfRecord(DamlList<RecordArgument>),
    TupleStructMapOfPrimitive(DamlTextMap<DamlParty>),
    TupleStructMapOfRecord(DamlTextMap<RecordArgument>),
    TupleStructOptionalOfPrimitive(DamlOptional<DamlBool>),
    TupleStructOptionalOfRecord(DamlOptional<RecordArgument>),
    TupleStructComplexType(DamlOptional<DamlList<DamlInt64>>),
    TupleStructRecord(RecordArgument),
    Record(AnonRecord),
}

#[DamlData]
pub struct RecordArgument {
    field_aaa: DamlInt64,
    field_bbb: DamlList<DamlText>,
}

#[DamlData]
pub struct AnonRecord {
    field_aaa: DamlInt64,
    field_bbb: DamlList<DamlText>,
}

#[DamlTemplate(package_id = "test", module_name = "DA.VariantExamples")]
pub struct VariantTemplate {
    pub owner: DamlParty,
    pub variants: DamlList<AllVariantTypes>,
}
