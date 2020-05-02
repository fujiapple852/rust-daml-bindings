use daml::prelude::*;

#[DamlData]
pub struct RGBA {
    pub r: DamlInt64,
    pub g: DamlInt64,
    pub b: DamlInt64,
    pub alpha: DamlInt64,
}

#[DamlVariant]
pub enum Color {
    Red,
    Green,
    Blue,
    Custom(DamlList<DamlInt64>),
    Other(RGBA),
}

#[DamlData]
pub struct Circle {
    pub radius: DamlNumeric10,
    pub color: Color,
}

#[DamlTemplate(package_id = r"test", module_name = "DA.Shape")]
pub struct CircleTemplate {
    pub owner: DamlParty,
    pub circle: Circle,
}

#[DamlChoices]
impl CircleTemplate {
    #[ReplaceCircle]
    pub fn replace_circle(new_circle: Circle) {}
}
