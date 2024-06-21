use std::fmt;
#[derive(Debug, Clone)]
pub struct ChatMessage {
    sender_address: std::net::SocketAddr,
    content: String,
}

impl ChatMessage {
    pub fn new(sender_addr : std::net::SocketAddr, content: String) -> ChatMessage {
        ChatMessage {
            sender_address : sender_addr,
            content,
        }
    }

    pub fn get_address(&self) -> std::net::SocketAddr {
        self.sender_address
    }

    pub fn get_content(&self) -> String {
        self.content.clone()
    }

}

impl fmt::Display for ChatMessage {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}: {}", self.sender_address, self.content)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_message_new() {
        let sender_addr = std::net::SocketAddr::new(std::net::IpAddr::V4(std::net::Ipv4Addr::new(127, 0, 0, 1)), 8080);
        let content = "Hello, Bob!".to_string();

        let chat_message: ChatMessage =
            ChatMessage::new(sender_addr, content.clone());

        // Assert to check if ChatMessage struct is constructed properly
        assert_eq!(chat_message.sender_address, sender_addr);
        assert_eq!(chat_message.content, content);

    }
    #[test]
    fn test_to_string() {
        let sender_addr = std::net::SocketAddr::new(std::net::IpAddr::V4(std::net::Ipv4Addr::new(127, 0, 0, 1)), 8080);
        let content = "Hello, Bob!".to_string();

        let chat_message: ChatMessage =
            ChatMessage::new(sender_addr, content.clone());

        assert_eq!(chat_message.to_string(), format!("{}: {}", sender_addr, content));
    }

        #[test]
    fn test_get_content() {
        let sender_addr = std::net::SocketAddr::new(std::net::IpAddr::V4(std::net::Ipv4Addr::new(127, 0, 0, 1)), 8080);
        let content = "Hello, Bob!".to_string();

        let chat_message: ChatMessage =
            ChatMessage::new(sender_addr, content.clone());

        assert_eq!(chat_message.get_content(), content);
    }

            #[test]
    fn test_get_sender_address() {
        let sender_addr = std::net::SocketAddr::new(std::net::IpAddr::V4(std::net::Ipv4Addr::new(127, 0, 0, 1)), 8080);
        let content = "Hello, Bob!".to_string();

        let chat_message: ChatMessage =
            ChatMessage::new(sender_addr, content.clone());

        assert_eq!(chat_message.get_address(), sender_addr);
    }

}
