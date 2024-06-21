use crate::chat_menu;
use crate::helper_functions;
use crate::login;
use crate::structs::chat_room::ChatRoom;
use crate::structs::user::User;
use colored::Colorize;
use crossterm::terminal::Clear;
use futures_util::stream::StreamExt;
use futures_util::SinkExt;
use http::Uri;
use std::io::Write;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio_websockets::{ClientBuilder, MaybeTlsStream, Message, WebSocketStream};

pub enum ClientState {
    AuthenticationMenu,
    ChatMenu(User),
    ChatRoom(User, ChatRoom),
    Exit,
}

/// Method for the client to authenticate with the server
async fn authenticate() -> Result<ClientState, Box<dyn std::error::Error>> {
    let mut current_user: Option<User>;
    loop {
        current_user = login::start_authentication_process_for_client().await;
        match current_user.clone() {
            None => {
                continue;
            }
            Some(_user) => {
                break;
            }
        }
    }
    Ok(ClientState::ChatMenu(current_user.unwrap().clone()))
}

// Method for the client to select a chat room
async fn client_main_menu(current_user: User) -> Result<ClientState, Box<dyn std::error::Error>> {
    let selected_chatroom = chat_menu::chat_menu_loop(current_user.clone()).await;
    match selected_chatroom {
        Some(chatroom) => Ok(ClientState::ChatRoom(current_user, chatroom)),
        None => Ok(ClientState::AuthenticationMenu),
    }
}

/// Method for the client to join a chat room and chat with a friend
async fn chat_room(
    current_user: User,
    selected_chatroom: ChatRoom,
) -> Result<ClientState, Box<dyn std::error::Error>> {
    let mut ws_stream = create_websocket_connection().await?;

    // CLear the terminal screen and print the chat room name
    _ = helper_functions::clear_console();

    // Print the welcoming message
    let welcome_msg = format!(
        "Welcome to the chat room: {}.",
        selected_chatroom.get_name()
    );
    println!("{}", welcome_msg.bold().green());
    let info_msg = "Type a message and press ENTER to send.\n(leave blank to return)";
    println!("{}", info_msg.yellow());

    // Restore chat history
    _ = selected_chatroom
        .clone()
        .print_chat_history(current_user.get_name().clone())
        .await;

    // Send the chat room id to the server
    ws_stream
        .send(Message::text(selected_chatroom.get_id().to_string()))
        .await?;

    // Main loop for chat room
    let stdin = tokio::io::stdin();
    let mut stdin = BufReader::new(stdin).lines();

    loop {
        tokio::select! {
            incoming = ws_stream.next() => {
                match incoming {
                    Some(Ok(msg)) => {
                        if let Some(text) = msg.as_text() {
                            // Handle incoming message
                            crossterm::execute!(std::io::stdout(), crossterm::cursor::MoveToColumn(0), Clear(crossterm::terminal::ClearType::CurrentLine)).unwrap();

                            // Split msg into username and message
                            let split_msg: Vec<&str> = text.splitn(2, ": ").collect();
                            let username = split_msg[0];
                            let message = split_msg[1];
                            println!("{}: {}", username.bold().cyan(), message);
                        }
                    },
                    Some(Err(err)) => return Err(err.into()),
                    None => return Ok(ClientState::Exit), // Connection closed
                }
            }
            res = {
                print!("{}: ", current_user.get_name().clone().bold().purple());
                std::io::stdout().flush().unwrap();
                stdin.next_line()
            } => {
                match res {
                    Ok(None) => return Ok(ClientState::Exit),
                    Ok(Some(line)) => {
                        if line.is_empty() {
                            return Ok(ClientState::ChatMenu(current_user));
                        }
                        let formatted_msg = format!("{}: {}", current_user.get_name(), line);
                        ws_stream.send(Message::text(formatted_msg)).await?;
                    }
                    Err(err) => return Err(err.into()),
                }
            }

        }
    }
}

/// Method for creating the
async fn create_websocket_connection(
) -> Result<WebSocketStream<MaybeTlsStream<tokio::net::TcpStream>>, Box<dyn std::error::Error>> {
    let (ws_stream, _) = ClientBuilder::from_uri(Uri::from_static("ws://127.0.0.1:8000"))
        .connect()
        .await?;
    Ok(ws_stream)
}

/// Entry method for the client
pub async fn run(mut current_state: ClientState) -> Result<(), Box<dyn std::error::Error>> {
    loop {
        current_state = match current_state {
            ClientState::AuthenticationMenu => authenticate().await?,
            ClientState::ChatMenu(user) => client_main_menu(user).await?,
            ClientState::ChatRoom(user, chatroom) => chat_room(user, chatroom).await?,
            ClientState::Exit => {
                println!("Exiting application...");
                break;
            }
        };
    }

    Ok(())
}