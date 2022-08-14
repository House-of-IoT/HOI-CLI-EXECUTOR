use std::collections::HashSet;

use serde_json::json;

use crate::parser::{accepted_commands, require_data_with_command, super_auth_commands};

pub struct MainState {
    pub admin_password: String,
    pub regular_password: String,
    pub super_admin_password: String,
    pub name: String,
    pub outside_name: String,
    pub connection_str: String,
    pub accepted_commands: HashSet<String>,
    pub super_auth_commands: HashSet<String>,
    pub commands_that_requires_data: HashSet<String>,
}

impl MainState {
    pub fn new(
        admin_password: String,
        regular_password: String,
        super_admin_password: String,
        name: String,
        outside_name: String,
        connection_str: String,
    ) -> Self {
        Self {
            admin_password,
            regular_password,
            super_admin_password,
            name,
            outside_name,
            connection_str,
            accepted_commands: accepted_commands(),
            super_auth_commands: super_auth_commands(),
            commands_that_requires_data: require_data_with_command(),
        }
    }
    pub fn name_and_type(&self) -> String {
        serde_json::to_string(&json!({"name":self.name,"type":"non-bot"})).unwrap()
    }
}
