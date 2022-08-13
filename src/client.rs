
use futures_channel::mpsc::UnboundedSender;
use futures_util::{stream::SplitStream, StreamExt};
use std::sync::Arc;
use tokio::{net::TcpStream, sync::RwLock};
use tokio_tungstenite::{
    connect_async, tungstenite::protocol::Message, MaybeTlsStream, WebSocketStream,
};

use crate::{request_types::HOIActionData, state::MainState};

pub async fn connect_and_begin_listening(server_state: Arc<RwLock<MainState>>) {
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
        } else {
        }
    }
}

pub async fn execute_action(tx: &mut UnboundedSender<Message>, action_data: HOIActionData) {
    // Normal HOI bot control protocol
    tx.unbounded_send(Message::text("bot_control".to_owned()))
        .unwrap_or_default();
    tx.unbounded_send(Message::text(action_data.action))
        .unwrap_or_default();
    tx.unbounded_send(Message::text(action_data.bot_name))
        .unwrap_or_default();
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
                println!("{}", msg);
                if msg.is_text() && msg.to_string() == "success" {
                    return true;
                }
            }
        };
    }
    false
}
