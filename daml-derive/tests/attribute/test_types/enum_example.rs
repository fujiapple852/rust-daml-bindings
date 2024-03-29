use daml::prelude::*;

#[DamlEnum]
pub enum SimpleColor {
    Red,
    Green,
    Blue,
}

#[DamlTemplate(package_id = r"test", module_name = "Fuji.Vehicle")]
pub struct Car {
    pub owner: DamlParty,
    pub driver: DamlParty,
    pub make: DamlText,
    pub color: SimpleColor,
    pub reg_year: DamlDate,
    pub purchase_time: DamlTimestamp,
}

#[DamlChoices]
impl Car {
    #[Repaint]
    pub fn repaint(new_color: SimpleColor) {}
}
