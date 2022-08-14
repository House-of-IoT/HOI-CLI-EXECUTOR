use anyhow::Result;
use async_std::io;

use futures_util::stream::SplitStream;
use serde_json::Value;
use std::{collections::HashSet, io::Write, sync::Arc};
use tokio::{net::TcpStream, sync::RwLock};
use tokio_tungstenite::{tungstenite::protocol::Message, MaybeTlsStream, WebSocketStream};

use crate::{
    client::{execute_action, execute_request},
    state::MainState,
};
#[derive(PartialEq)]
enum OutputStyle {
    Pretty,
    Raw,
}

pub async fn gather_input_and_route(
    tx: &mut futures_channel::mpsc::UnboundedSender<Message>,
    read: &mut SplitStream<WebSocketStream<MaybeTlsStream<TcpStream>>>,
    server_state: &Arc<RwLock<MainState>>,
) -> Result<()> {
    let input = gather_input().await;
    if let Ok(input_string) = input {
        let command_parameters: Vec<String> = input_string.split(" ").map(String::from).collect();
        // Our commands only follow the following format:
        // -> command_op_code/ data display type
        // -> command_op_code/ data for the request/ data display type
        // -> command_op_code
        //
        // Some requests need data to go along with them and some don't
        // and the user can specify which way they would like their data
        // to be displayed, "pretty"(default) or "raw".
        //
        // pretty -> Has an effect on json based responses
        // raw -> Raw, with no formatting in the terminal.
        if command_parameters.len() > 0 && command_parameters.len() < 4 {
            // Is the command valid?
            if server_state
                .read()
                .await
                .accepted_commands
                .contains(&command_parameters[0])
            {
                let print_style =
                    match command_parameters.len() == 3 && command_parameters[2] == "raw" {
                        true => OutputStyle::Raw,
                        false => OutputStyle::Pretty,
                    };
                let response =
                    command_to_functionality(command_parameters, tx, read, server_state).await?;
                let output_result = format_final_result(response, print_style);
                println!("{}", output_result);
                println!(" ");
            }
        }
    }
    Ok(())
}

async fn gather_input() -> Result<String> {
    print!("$->");
    std::io::stdout().flush().unwrap();
    let stdin = io::stdin();
    let mut line = String::new();
    stdin.read_line(&mut line).await?;
    println!("  ");
    Ok(line.trim().to_owned())
}

pub fn accepted_commands() -> HashSet<String> {
    HashSet::from([
        "control".to_owned(),
        "devices".to_owned(),
        "deactivated".to_owned(),
        "banned".to_owned(),
        "passive_data".to_owned(),
        "external_controller".to_owned(),
        "custom_type_add".to_owned(),
        "add-task".to_owned(),
        "remove-task".to_owned(),
        "add-contact".to_owned(),
        "remove-contact".to_owned(),
        "add-banned".to_owned(),
        "remove-banned".to_owned(),
        "contacts".to_owned(),
        "recent_connections".to_owned(),
        "executed_actions".to_owned(),
        "server_config".to_owned(),
        "recent_executed_tasks".to_owned(),
    ])
}

pub fn super_auth_commands() -> HashSet<String> {
    HashSet::from([
        "add-banned-ip".to_owned(),
        "remove-banned-ip".to_owned(),
        "add-task".to_owned(),
        "remove-task".to_owned(),
        "remove-contact".to_owned(),
        "add-contact".to_owned(),
    ])
}

pub fn require_data_with_command() -> HashSet<String> {
    HashSet::from([
        "custom_type_add".to_owned(),
        "control".to_owned(),
        "external_controller".to_owned(),
        "add-banned-ip".to_owned(),
        "remove-banned-ip".to_owned(),
        "add-task".to_owned(),
        "remove-task".to_owned(),
        "remove-contact".to_owned(),
        "add-contact".to_owned(),
    ])
}

fn command_to_op_code(code: &str) -> String {
    match code {
        "control" => "bot_control".to_owned(),
        "devices" => "servers_devices".to_owned(),
        "deactivated" => "servers_deactivated_bots".to_owned(),
        "banned" => "servers_banned_ips".to_owned(),
        "external_controller" => "external_controller_request".to_owned(),
        "remove-banned" => "remove-banned-ip".to_owned(),
        "add-banned" => "add-banned-ip".to_owned(),
        _ => code.to_owned(),
    }
}

async fn command_to_functionality(
    command_parameters: Vec<String>,
    tx: &mut futures_channel::mpsc::UnboundedSender<Message>,
    read: &mut SplitStream<WebSocketStream<MaybeTlsStream<TcpStream>>>,
    server_state: &Arc<RwLock<MainState>>,
) -> Result<String> {
    if server_state
        .read()
        .await
        .commands_that_requires_data
        .contains(&command_parameters[0])
        && command_parameters.len() >= 2
    {
        if command_parameters[0] == "control" {
            if let Ok(data) = serde_json::from_str(&command_parameters[1]) {
                return execute_action(tx, read, data).await;
            }
        }
    }
    let request_data = match command_parameters.len() == 2 {
        true => Some(command_parameters[1].clone()),
        false => None,
    };
    let routed_op_code = command_to_op_code(&command_parameters[0]);
    return execute_request(routed_op_code, request_data, server_state, tx, read).await;
}

fn format_final_result(data: String, style: OutputStyle) -> String {
    if style == OutputStyle::Raw {
        return data;
    }
    if let Ok(data) = serde_json::from_str(&data) {
        let annotated_data: Value = data;
        let pretty_data = serde_json::to_string_pretty(&annotated_data);
        if let Ok(pretty_data) = pretty_data {
            return pretty_data;
        }
    }
    data
}
