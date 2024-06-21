use crate::client::ClientState;
use std::{env, process::exit};
use tokio::runtime::Runtime;

extern crate block_padding;

mod chat_menu;
mod client;
mod helper_functions;
mod login;
mod server;
mod sql_interaction;

mod structs {
    pub mod chat_room;
    pub mod friend_request;
    pub mod message;
    pub mod user;
}

fn main() {
    let rt = Runtime::new().unwrap();
    rt.block_on(async {
        let args: Vec<String> = env::args().collect();
        if args.len() > 1 && args[1] == "server" {
            match server::run().await {
                Ok(_) => println!("Server exited successfully."),
                Err(e) => eprintln!("Server exited with error: {}", e),
            }
            exit(0);
        } else {
            let current_state = ClientState::AuthenticationMenu;
            match client::run(current_state).await {
                Ok(_) => println!("Client exited successfully."),
                Err(e) => eprintln!("Client exited with error: {}", e),
            }
            exit(0);
        }
    });
}
