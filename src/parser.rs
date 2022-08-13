use anyhow::Result;
use async_std::io;

use futures_util::stream::SplitStream;
use std::{io::Write, sync::Arc};
use tokio::{net::TcpStream, sync::RwLock};
use tokio_tungstenite::{tungstenite::protocol::Message, MaybeTlsStream, WebSocketStream};

use crate::state::MainState;

async fn gather_input_and_route(
    tx: &mut futures_channel::mpsc::UnboundedSender<Message>,
    read: &mut SplitStream<WebSocketStream<MaybeTlsStream<TcpStream>>>,
    server_state: &Arc<RwLock<MainState>>,
) {
    let input = gather_input().await;
    if let Ok(input_string) = input {
        let command_parameters: Vec<&str> = input_string.split(" ").collect();
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

    }
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
