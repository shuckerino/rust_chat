use std::sync::{Arc, RwLock};
use tokio::sync::broadcast::error::SendError;

use crate::{sql_interaction, structs::message::ChatMessage};
use colored::Colorize;
use tokio::sync::broadcast::{channel, Receiver, Sender};

#[derive(Debug, Clone)]
pub struct ChatRoom {
    id: u32,
    name: String,
    user1_id: u32,
    user2_id: u32,
    bcast_sender: Sender<ChatMessage>,
    bcast_receiver: Arc<RwLock<Receiver<ChatMessage>>>,
}

impl ChatRoom {
    pub fn new(id: u32, name: String, user1_id: u32, user2_id: u32) -> Self {
        let (bcast_tx, bcast_rx) = channel::<ChatMessage>(16);
        ChatRoom {
            id,
            name,
            user1_id,
            user2_id,
            bcast_sender: bcast_tx,
            bcast_receiver: Arc::new(RwLock::new(bcast_rx)),
        }
    }

    pub fn get_name(&self) -> &String {
        &self.name
    }

    pub fn get_id(&self) -> &u32 {
        &self.id
    }

    pub fn get_sender(&self) -> Sender<ChatMessage> {
        self.bcast_sender.clone()
    }

    pub fn get_receiver(&self) -> Arc<RwLock<Receiver<ChatMessage>>> {
        self.bcast_receiver.clone()
    }

    pub fn get_user1_id(&self) -> u32 {
        self.user1_id
    }

    pub fn get_user2_id(&self) -> u32 {
        self.user2_id
    }

    pub fn broadcast_message(&self, message: ChatMessage) -> Result<usize, SendError<ChatMessage>> {
        self.bcast_sender.send(message)
    }

    pub async fn print_chat_history(
        &mut self,
        current_client: String,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let msgs = sql_interaction::get_chat_messages_for_chatroom_from_database(self.id).await;

        for msg in msgs.as_ref().unwrap() {
            // Split the message into sender and content
            let parts: Vec<String> = msg.splitn(2, ':').map(|s| s.to_string()).collect();

            if parts.len() == 2 {
                let sender = parts[0].trim();
                let content = parts[1].trim();

                // Check if the current client sent the message
                if sender == current_client {
                    println!("{}: {}", sender.bold().purple(), content);
                } else {
                    println!("{}: {}", sender.bold().cyan(), content);
                }
            } else {
                eprintln!("Message does not contain a colon: {}", msg);
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_chatroom_name() {
        let chatroom = ChatRoom::new(1, "Chatroom 1".to_string(), 1, 2);
        assert_eq!(*chatroom.get_name(), "Chatroom 1".to_string());
    }

    #[test]
    fn test_get_chatroom_id() {
        let chatroom = ChatRoom::new(1, "Chatroom 1".to_string(), 1, 2);
        assert_eq!(*chatroom.get_id(), 1);
    }

    #[test]
    fn test_get_sender() {
        let chatroom = ChatRoom::new(1, "Chatroom 1".to_string(), 1, 2);
        let sender = chatroom.get_sender();
        assert_eq!(sender.receiver_count(), 1);
    }

    #[test]
    fn test_broadcast_message_positive() {
        let chatroom = ChatRoom::new(1, "Chatroom 1".to_string(), 1, 2);
        let sender_addr = std::net::SocketAddr::new(
            std::net::IpAddr::V4(std::net::Ipv4Addr::new(127, 0, 0, 1)),
            8080,
        );
        let content = "Hello, Bob!".to_string();

        let chat_message: ChatMessage = ChatMessage::new(sender_addr, content.clone());
        let result = chatroom.broadcast_message(chat_message);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 1);
    }

    #[tokio::test]
    async fn test_print_chat_history_positive() {
        let mut chatroom = ChatRoom::new(1, "TestChat1".to_string(), 1, 2);
        let result = chatroom.print_chat_history("rino".to_string()).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_print_chat_history_missing_colon_in_message() {
        let mut chatroom = ChatRoom::new(2, "TestChat2".to_string(), 1, 3);
        let result = chatroom.print_chat_history("anton".to_string()).await;
        assert!(result.is_ok());
    }

    #[test]
    fn test_get_user1_id() {
        let chatroom = ChatRoom::new(1, "Chatroom 1".to_string(), 1, 2);
        assert_eq!(chatroom.get_user1_id(), 1);
    }

    #[test]
    fn test_get_user2_id() {
        let chatroom = ChatRoom::new(1, "Chatroom 1".to_string(), 1, 2);
        assert_eq!(chatroom.get_user2_id(), 2);
    }
}
