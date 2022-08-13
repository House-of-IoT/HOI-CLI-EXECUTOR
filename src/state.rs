use serde_json::json;

pub struct MainState {
    pub admin_password: String,
    pub regular_password: String,
    pub super_admin_password: String,
    pub name: String,
    pub outside_name: String,
    pub connection_str: String,
}

impl MainState {
    pub fn name_and_type(&self) -> String {
        serde_json::to_string(&json!({"name":self.name,"type":"non-bot"})).unwrap()
    }
}
