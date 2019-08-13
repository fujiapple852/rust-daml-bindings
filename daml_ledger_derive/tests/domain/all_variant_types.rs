use daml::prelude::*;

/// The following is an example of how this DAML variant data type can be represented:
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
#[DamlData]
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
}

#[DamlData]
pub struct RecordArgument {
    field_aaa: DamlInt64,
    field_bbb: DamlList<DamlText>,
}

#[DamlTemplate(
    package_id = "510e5612a7970a6d7615bc940e8ee6b4da3eb12f257e59268e729683e9929e8b",
    module_name = "DA.VariantExamples"
)]
pub struct VariantTemplate {
    pub owner: DamlParty,
    pub variants: DamlList<AllVariantTypes>,
}
