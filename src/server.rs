use crate::structs::chat_room::ChatRoom;
use crate::structs::message::ChatMessage;
use crate::sql_interaction;
use futures_util::sink::SinkExt;
use futures_util::stream::StreamExt;
use std::collections::HashMap;
use std::error::Error;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::Mutex;
use tokio_tungstenite::accept_async;
use tokio_tungstenite::tungstenite::protocol::Message as WsMessage;
use tokio_tungstenite::WebSocketStream;

/// Main Function for running the ChatRoom Server
pub async fn run() -> Result<(), Box<dyn Error + Send + Sync>> {
    // Bind the server to the address
    let addr = "127.0.0.1:8000".to_string();
    let listener = TcpListener::bind(addr).await?;
    println!("Server listening on port 8000");

    // Use Arc and Mutex to share the chat_rooms safely across threads
    let chat_rooms = Arc::new(Mutex::new(HashMap::<u32, ChatRoom>::new()));

    loop {
        let (stream, addr) = listener.accept().await?;
        println!("New connection from {:?}", addr);

        let chat_rooms: Arc<Mutex<HashMap<u32, ChatRoom>>> = Arc::clone(&chat_rooms);

        tokio::spawn(async move {
            match accept_async(stream).await {
                Ok(mut ws_stream) => {
                    if let Some(Ok(message)) = ws_stream.next().await {
                        if let Ok(chatroom_id) = message.to_text().unwrap().parse::<u32>() {
                            println!("Chat Id is: {:?}", chatroom_id);

                            let current_chatroom = {
                                let mut chat_rooms_lock = chat_rooms.lock().await;

                                if let Some(chatroom) = chat_rooms_lock.get(&chatroom_id) {
                                    println!("Chatroom already exists with id: {:?}, user1_id: {:?}, user2_id: {:?}",
                                             chatroom.get_id(), chatroom.get_user1_id(), chatroom.get_user2_id());
                                    chatroom.clone()
                                } else {
                                    let new_chatroom =
                                        sql_interaction::get_chatroom_by_id(chatroom_id)
                                            .await
                                            .unwrap();
                                    chat_rooms_lock.insert(chatroom_id, new_chatroom.clone());
                                    new_chatroom
                                }
                            };

                            // Handle the client connection
                            _ = handle_single_client_connection(addr, ws_stream, current_chatroom)
                                .await;
                        } else {
                            println!("Invalid Chat Id received");
                        }
                    } else {
                        println!("No Chat Id received or error in receiving message");
                    }
                }
                Err(e) => {
                    println!("Error accepting websocket connection: {:?}", e);
                }
            }
        });
    }
}

/// This function handles the connection for each client.
async fn handle_single_client_connection(
    addr: SocketAddr,
    mut ws_stream: WebSocketStream<TcpStream>,
    chatroom: ChatRoom,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    println!("Current count of receivers for this chatroom is: {:?}", chatroom.get_receiver().read().unwrap().len());
    let mut chatroom_receiver = chatroom.get_sender().subscribe();

    // A continuous loop for concurrently performing two tasks:
    //(1) receiving messages from `ws_stream` and broadcasting them
    //(2) receiving messages on `bcast_rx` and sending them to the client.
    loop {
        tokio::select! {
            incoming = ws_stream.next() => {
                match incoming {
                    Some(Ok(msg)) => {

                        if let Ok(msg) = msg.into_text(){
                            let custom_msg = ChatMessage::new(addr, msg.to_string());
                            println!("{}", custom_msg);
                            _ = chatroom.broadcast_message(custom_msg.clone());

                            // Spawn a task to handle the database interaction in the background
                            let chatroom_id = *chatroom.get_id();
                            let content = custom_msg.get_content().clone();
                            tokio::task::spawn(async move {
                                let result = sql_interaction::save_chat_message_to_database(chatroom_id, content).await;
                                match result {
                                    Ok(_) => println!("Message saved to database"),
                                    Err(e) => eprintln!("Error saving message to database: {}", e),
                                }
                            });


                        }
                        }
                        Some(Err(err)) => return Err(err.into()),
                        None => return Ok(()),  // Stream ended
                }
            }
            msg = chatroom_receiver.recv() => {
                if let Ok(chat_msg) = msg {

                    if chat_msg.get_address() != addr { // Don't send message back to the sender
                        ws_stream
                    .send(WsMessage::Text(chat_msg.get_content())).await?;
                    }
                }
            }
        }
    }
}
