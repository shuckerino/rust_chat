use super::{chat_room::ChatRoom, user::User};
use crate::sql_interaction;

#[derive(Debug, Clone)]
pub struct FriendRequest {
    sender_id: u32,
    _receiver_id: u32,
    _is_accepted: bool,
    display_name: String, // Equal to name of the sender
}

impl FriendRequest {
    // Constructor
    pub fn new(
        sender_id: u32,
        _receiver_id: u32,
        _is_accepted: bool,
        display_name: String,
    ) -> FriendRequest {
        FriendRequest {
            sender_id,
            _receiver_id,
            _is_accepted,
            display_name,
        }
    }

    pub fn get_display_name(&self) -> &String {
        &self.display_name
    }

    pub fn get_sender_id(&self) -> u32 {
        self.sender_id
    }

    pub async fn accept_friend_request(&self, receiving_user: User) -> Option<ChatRoom> {
        _ = sql_interaction::accept_friend_request_for_user(
            self.get_sender_id(),
            receiving_user.get_id(),
        )
        .await;

        _ = sql_interaction::create_friends(self.get_sender_id(), receiving_user.get_id()).await;

        let chatroom_name = format!(
            "{} and {}'s chat",
            receiving_user.get_name(),
            self.get_display_name()
        );
        // create new chatroom to show in "Join an existing chatroom"
        let _ = sql_interaction::create_new_chatroom(
            receiving_user.get_id(),
            self.get_sender_id(),
            chatroom_name,
        )
        .await;
        None // theoretically, we could return the chatroom directly to immediately join it, but instead we return to the chat menu
    }

    pub async fn decline_friend_request(
        &self,
        current_user_id: u32,
    ) -> Result<(), Box<dyn std::error::Error>> {
        _ = sql_interaction::delete_friend_request(self.get_sender_id(), current_user_id).await;
        println!("Friend request declined!");

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_display_name() {
        let friend_request = FriendRequest::new(1, 2, false, "Alice".to_string());
        assert_eq!(*friend_request.get_display_name(), "Alice".to_string());
    }

    #[test]
    fn test_get_sender_id() {
        let friend_request = FriendRequest::new(1, 2, false, "Alice".to_string());
        assert_eq!(friend_request.get_sender_id(), 1);
    }

    #[tokio::test]
    async fn test_accepting_friend_request() {
        // Accept rinos request to antonia
        let antonia = User::new(4, "Antonia".to_string());
        let friend_request = FriendRequest::new(2, 4, false, "rino".to_string());
        let chatroom = friend_request.accept_friend_request(antonia.clone()).await;
        assert!(chatroom.is_none());
    }

    #[tokio::test]
    async fn test_declining_friend_request() {
        // Declining rinos friend request to testuser
        // let rino = sql_interaction::get_user_from_database_by_id(2).await.unwrap();
        let friend_request = FriendRequest::new(2, 3, false, "rino".to_string());
        let res = friend_request.decline_friend_request(3).await;
        assert!(res.is_ok());
    }
}
