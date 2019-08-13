use daml::prelude::*;

#[DamlData]
pub struct UserData {
    pub name: DamlParty,
    pub new_value: DamlInt64,
}

#[DamlTemplate(
    package_id = r"510e5612a7970a6d7615bc940e8ee6b4da3eb12f257e59268e729683e9929e8b",
    module_name = "DA.PingPong"
)]
pub struct Ping {
    pub sender: DamlParty,
    pub receiver: DamlParty,
    pub count: DamlInt64,
}

#[DamlChoices]
impl Ping {
    #[RespondPong]
    fn respond_pong(&self) {}

    #[ResetPingCount]
    fn reset_ping_count(&self) {}

    #[FromUserData]
    fn set_from_user_data(&self, new_count: DamlInt64, new_data: UserData) {}
}

#[DamlTemplate(
    package_id = r"510e5612a7970a6d7615bc940e8ee6b4da3eb12f257e59268e729683e9929e8b",
    module_name = "DA.PingPong"
)]
pub struct Pong {
    pub sender: DamlParty,
    pub receiver: DamlParty,
    pub count: DamlInt64,
}

#[DamlChoices]
impl Pong {
    #[RespondPong]
    fn respond_ping(&self, new_count: DamlInt64) {}

    #[ResetPongCount]
    fn reset_pong_count(&self) {}
}
