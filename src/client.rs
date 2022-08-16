use anyhow::Result;
use futures_channel::mpsc::UnboundedSender;
use futures_util::{stream::SplitStream, StreamExt};
use serde_json::Value;
use std::sync::Arc;
use tokio::{net::TcpStream, sync::RwLock};
use tokio_tungstenite::{
    connect_async, tungstenite::protocol::Message, MaybeTlsStream, WebSocketStream,
};

use crate::{
    logging::console::{clear_terminal, log_event},
    parser::gather_input_and_route,
    request_types::HOIActionData,
    state::MainState,
};

pub async fn connect_and_begin_listening(server_state: Arc<RwLock<MainState>>) -> Result<()> {
    let url_res = url::Url::parse(&server_state.read().await.connection_str);
    println!("Connecting...");
    if let Ok(url) = url_res {
        let (mut stdin_tx, stdin_rx) = futures_channel::mpsc::unbounded();

        let (ws_stream, _) = connect_async(url).await.expect("Failed to connect");
        let (write, mut read) = ws_stream.split();
        let stdin_to_ws = stdin_rx.map(Ok).forward(write);
        tokio::task::spawn(stdin_to_ws);

        let is_authed = authenticate(&mut stdin_tx, &mut read, &server_state).await;

        if is_authed {
            loop {
                gather_input_and_route(&mut stdin_tx, &mut read, &server_state).await?;
            }
        }
    }
    Ok(())
}

pub async fn execute_action(
    tx: &mut UnboundedSender<Message>,
    read: &mut SplitStream<WebSocketStream<MaybeTlsStream<TcpStream>>>,
    action_data: HOIActionData,
) -> Result<String> {
    // Normal HOI bot control protocol
    tx.unbounded_send(Message::text("bot_control".to_owned()))?;
    tx.unbounded_send(Message::text(action_data.action))?;
    tx.unbounded_send(Message::text(action_data.bot_name))?;
    Ok(gather_ws_response(read).await)
}

pub async fn authenticate(
    tx: &mut futures_channel::mpsc::UnboundedSender<Message>,
    read: &mut SplitStream<WebSocketStream<MaybeTlsStream<TcpStream>>>,
    server_state: &Arc<RwLock<MainState>>,
) -> bool {
    let read_state = server_state.read().await;

    let password_send = tx.unbounded_send(Message::Text(read_state.regular_password.clone()));
    let name_and_type_send = tx.unbounded_send(Message::Text(read_state.name_and_type()));
    let outside_name_send = tx.unbounded_send(Message::Text(read_state.outside_name.clone()));
    if password_send.is_ok() && name_and_type_send.is_ok() && outside_name_send.is_ok() {
        if let Some(msg) = read.next().await {
            if let Ok(msg) = msg {
                if msg.is_text() && msg.to_string() == "success" {
                    clear_terminal();
                    log_event("Connected To Server");
                    return true;
                }
            }
        };
    }
    false
}

/// Executes request that requires one
/// set of data to be sent after the first
/// op code
pub async fn execute_request(
    request_op_code: String,
    request_data: Option<String>,
    server_state: &Arc<RwLock<MainState>>,
    tx: &mut futures_channel::mpsc::UnboundedSender<Message>,
    read: &mut SplitStream<WebSocketStream<MaybeTlsStream<TcpStream>>>,
) -> Result<String> {
    // Step one in the protocol send the request op code
    tx.unbounded_send(Message::text(request_op_code.clone()))
        .unwrap_or_default();

    // Step two in the protocol send the data for the request
    if let Some(request_data) = request_data {
        tx.unbounded_send(Message::text(request_data))
            .unwrap_or_default();
    }
    // Step Three in the protocol check authenticate if needed
    if let Some(Ok(msg)) = read.next().await {
        if msg.is_text() {
            let response = msg.to_string();
            if requires_admin_auth(response.clone()) {
                let password_for_auth =
                    route_opcode_to_auth_password(server_state, &request_op_code).await;
                return admin_authenticate_and_gather_response(tx, read, password_for_auth).await;
            } else {
                return Ok(response);
            }
        }
    }
    Ok("Issue With Execution".to_owned())
}
async fn admin_authenticate_and_gather_response(
    tx: &mut futures_channel::mpsc::UnboundedSender<Message>,
    read: &mut SplitStream<WebSocketStream<MaybeTlsStream<TcpStream>>>,
    password_for_auth: String,
) -> Result<String> {
    tx.unbounded_send(Message::text(password_for_auth))?;
    Ok(gather_ws_response(read).await)
}

/// Certain actions require different
/// authentication passwords, so when the
/// server requires an admin authetication
/// we need the correct password
async fn route_opcode_to_auth_password(
    server_state: &Arc<RwLock<MainState>>,
    code: &str,
) -> String {
    if server_state.read().await.super_auth_commands.contains(code) {
        server_state.read().await.super_admin_password.clone()
    } else {
        server_state.read().await.admin_password.clone()
    }
}

fn requires_admin_auth(response: String) -> bool {
    if let Ok(response_from_server) = serde_json::from_str(&response) {
        let actual_response: Value = response_from_server;
        if actual_response["status"] != Value::Null
            && actual_response["status"]
                .to_string()
                .contains("needs-admin-auth")
        {
            return true;
        }
    }

    false
}

async fn gather_ws_response(
    read: &mut SplitStream<WebSocketStream<MaybeTlsStream<TcpStream>>>,
) -> String {
    if let Some(Ok(msg)) = read.next().await {
        if msg.is_text() {
            return msg.to_string();
        }
    }
    "Issue During Message Gather".to_owned()
}
