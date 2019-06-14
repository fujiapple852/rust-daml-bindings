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
    package_id = r"045a30fb3e25804277456215a9bd7b8d93406e62a87ac1c07f6aeb7c9e1ca066",
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
