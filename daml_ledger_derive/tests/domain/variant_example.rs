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
    package_id = r"510e5612a7970a6d7615bc940e8ee6b4da3eb12f257e59268e729683e9929e8b",
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
