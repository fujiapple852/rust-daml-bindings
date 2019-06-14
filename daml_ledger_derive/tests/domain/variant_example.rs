use daml::prelude::*;

#[DamlData]
pub struct RGBA {
    pub r: DamlInt64,
    pub g: DamlInt64,
    pub b: DamlInt64,
    pub alpha: DamlInt64,
}

#[DamlData]
pub enum Color {
    Red,
    Green,
    Blue,
    Custom(DamlList<DamlInt64>),
    Other(RGBA),
}

#[DamlData]
pub struct Circle {
    pub radius: DamlDecimal,
    pub color: Color,
}

#[DamlTemplate(
    package_id = r"6ff89900a3badb67b538c6be4e4ca3adba7653d8f28b6af4aeac02bfad517fdb",
    module_name = "DA.Shape"
)]
pub struct CircleTemplate {
    pub owner: DamlParty,
    pub circle: Circle,
}

#[DamlChoices]
impl CircleTemplate {
    #[ReplaceCircle]
    pub fn replace_circle(new_circle: Circle) {}
}
