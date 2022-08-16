use client::connect_and_begin_listening;
use logging::console::clear_terminal;
use state::MainState;
use std::io::stdin;
use std::io::Write;
use std::sync::Arc;
use tokio::sync::RwLock;
mod client;
pub mod logging {
    pub mod console;
}
mod parser;
mod request_types;
mod state;
#[tokio::main]
async fn main() {
    clear_terminal();
    let admin_password = gather_blocking_input("Admin Password->");
    let reg_password = gather_blocking_input("Regular Password->");
    let super_admin_password = gather_blocking_input("Super Admin Password->");
    let name = gather_blocking_input("Name->");
    let outside_name = gather_blocking_input("Outside Name->");
    let connection_str = gather_blocking_input("Connection String->");

    let state = Arc::new(RwLock::new(MainState::new(
        admin_password,
        reg_password,
        super_admin_password,
        name,
        outside_name,
        connection_str,
    )));
    loop {
        connect_and_begin_listening(state.clone())
            .await
            .unwrap_or_default();
    }
}

fn gather_blocking_input(prompt: &str) -> String {
    let mut input: String = String::new();
    print!("{}", prompt);
    std::io::stdout().flush().unwrap();
    stdin().read_line(&mut input).unwrap();
    println!("  ");
    input.trim().to_string()
}
