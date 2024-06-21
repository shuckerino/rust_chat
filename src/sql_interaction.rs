use crate::helper_functions;
use crate::structs::chat_room::ChatRoom;
use crate::structs::friend_request::FriendRequest;
use crate::structs::user::User;
use lazy_static::lazy_static;
use mysql::*;
use prelude::Queryable;
use std::error::Error;
use tokio::sync::Mutex;

lazy_static! {
    static ref DB_POOL: Mutex<Option<Pool>> = Mutex::new(None);
}
fn initialize_db_pool() -> Result<Pool, Box<dyn Error>> {
    let url = "mysql://root:root@10.8.13.39:47777/chatclient";
    let pool = Pool::new(url)?;
    Ok(pool)
}

async fn get_dbconn() -> Result<PooledConn, Box<dyn Error>> {
    let mut db_pool = DB_POOL.lock().await;

    // If the pool is not initialized, initialize it
    if db_pool.is_none() {
        let pool = initialize_db_pool()?;
        *db_pool = Some(pool);
    }

    // Now we can safely unwrap the pool and get a connection
    match db_pool.as_ref().unwrap().get_conn() {
        Ok(conn) => Ok(conn),
        Err(err) => Err(Box::new(err)),
    }
}

///Returns true if a username exists in database
pub async fn check_if_username_exists(name: &str) -> Result<bool, Box<dyn std::error::Error>> {
    let mut user_exists = false;
    let mut conn = get_dbconn().await?.unwrap();
    let mut selected_usernames = Vec::new();

    //Select names of all entries in User table and add them to selected_usernames
    conn.query_map("SELECT UserName from users", |name: String| {
        selected_usernames.push(name);
    })?;

    if selected_usernames.contains(&String::from(name)) {
        user_exists = true;
    }

    Ok(user_exists)
}

///Checks if the entered password matches with the password saved in the database for a user with the username
pub async fn check_if_password_matches_username(
    password: &str,
    username: &str,
) -> Result<bool, Box<dyn Error>> {
    //Connect to database
    let mut conn = get_dbconn().await?.unwrap();

    // Prepare the query
    let query =
        r"SELECT Id, UserName FROM users WHERE UserName = :username AND Password = :password";

    // Execute the query
    let result = conn.exec_map(
        query,
        params! {
            "username" => username,
            "password" => password,
        },
        |(id, name)| User::new(id, name),
    )?;

    //Return true if received unique result
    Ok(result.len() == 1)
}

///Saves a new user with the username und the password that is hashed already to database
pub async fn save_new_user_to_database_after_signup(
    username: &String,
    password: &String,
) -> Result<(), Box<dyn Error>> {
    //Connect to database
    let mut conn = get_dbconn().await?.unwrap();

    // Prepare and execute the insert query
    conn.exec_drop(
        r"INSERT INTO users (UserName, Password, BlockedUntil) VALUES (:username, :password, :blocked)",
        params! {
            "username" => username,
            "password" => password,
            "blocked" => 0
        },
    )?;

    Ok(())
}

/// Method to get a user from the database by username
pub async fn get_user_from_database_by_name(username: String) -> Result<User, Box<dyn Error>> {
    //Connect to database
    let mut conn = get_dbconn().await?.unwrap();

    // Prepare the query
    let query = r"SELECT Id, UserName FROM users WHERE UserName = :username";

    // Execute the query
    let result = conn.exec_map(
        query,
        params! {
            "username" => username,
        },
        |(id, name)| User::new(id, name),
    )?;

    //Return the user if received unique result
    match result.len() {
        1 => Ok(result[0].clone()),
        _ => Err("User not found".into()),
    }
}

///Blocks user for a certain amount of time
pub async fn set_user_isblocked(username: String, duration: u64) -> Result<(), Box<dyn Error>> {
    // Connect to database
    let mut conn = get_dbconn().await?.unwrap();

    // Prepare and execute the update query
    conn.exec_drop(
        "UPDATE users SET BlockedUntil = ? WHERE UserName = ?",
        (
            helper_functions::get_sys_time_in_secs() + duration,
            username,
        ),
    )?;

    Ok(())
}

///Returns how long a user is still blocked
pub async fn get_user_is_blockeduntil(username: String) -> Result<u64, Box<dyn Error>> {
    //Connect to database
    let mut conn = get_dbconn().await?.unwrap();

    // Prepare and execute the select query
    let result: Option<u64> = conn.exec_first(
        r"SELECT BlockedUntil FROM users WHERE UserName = :username",
        params! {
            "username" => username,
        },
    )?;

    // Return the result, or false if no result was found
    match result {
        Some(blocked_until) => {
            //User is blocked if BlockedUntil time is larger that current time
            Ok(blocked_until)
        }
        None => Ok(0), // or handle as needed
    }
}

///Returns if a user is currently blocked or not
pub async fn check_if_user_isblocked(username: String) -> Result<bool, Box<dyn Error>> {
    //Connect to database
    let mut conn = get_dbconn().await?.unwrap();

    // Prepare and execute the select query
    let result: Option<u64> = conn.exec_first(
        r"SELECT BlockedUntil FROM users WHERE UserName = :username",
        params! {
            "username" => username,
        },
    )?;

    // Return true if the user is blocked, or false if no result was found
    match result {
        Some(blocked_until) => {
            //User is blocked if BlockedUntil time is larger that current time
            Ok(blocked_until > helper_functions::get_sys_time_in_secs())
        }
        None => Ok(false), // or handle as needed
    }
}

///Searches for usernames in the database that match the search string
pub async fn get_user_by_name_with_contains_search(
    search_string: String,
    current_user_id: u32,
) -> Result<Vec<User>, Box<dyn Error>> {
    //Connect to database
    let mut conn = get_dbconn().await?.unwrap();

    // For getting contains search
    let search_pattern = format!("%{}%", search_string);

    // Prepare the query
    let query = r"
        SELECT users.Id, users.UserName 
        FROM users 
        LEFT JOIN friends 
        ON (users.Id = friends.User1_Id AND friends.User2_Id = :current_user_id) 
        OR (users.Id = friends.User2_Id AND friends.User1_Id = :current_user_id)
        WHERE users.UserName LIKE :search_string 
        AND users.Id != :current_user_id
        AND friends.Id IS NULL";

    // Execute the query
    let result: Vec<User> = conn.exec_map(
        query,
        params! {
            "search_string" => search_pattern,
            "current_user_id" => current_user_id,
        },
        |(id, name)| User::new(id, name),
    )?;

    Ok(result)
}

pub async fn search_for_chatrooms_of_user(user_id: u32) -> Result<Vec<ChatRoom>, Box<dyn Error>> {
    //Connect to database
    let mut conn = get_dbconn().await?.unwrap();

    // Prepare the query
    let query = r"SELECT Id, ChatName, User1_Id, User2_Id FROM chats WHERE User1_Id = :user_id OR User2_Id = :user_id";

    // Execute the query
    let result: Vec<ChatRoom> = conn.exec_map(
        query,
        params! {
            "user_id" => user_id,
        },
        |(id, name, user1_id, user2_id)| ChatRoom::new(id, name, user1_id, user2_id),
    )?;

    Ok(result)
}

pub async fn create_new_chatroom(
    user1_id: u32,
    user2_id: u32,
    chatroom_name: String,
) -> Result<ChatRoom, Box<dyn Error>> {
    //Connect to database
    let mut conn = get_dbconn().await?.unwrap();

    // Prepare the query
    let query = r"INSERT INTO chats (ChatName, User1_Id, User2_Id) VALUES (:chatname, :user1_id, :user2_id)";

    // Execute the query
    let chatroom = conn.exec_map(
        query,
        params! {
            "chatname" => chatroom_name,
            "user1_id" => user1_id,
            "user2_id" => user2_id,
        },
        |(id, name, user1_id, user2_id)| ChatRoom::new(id, name, user1_id, user2_id),
    )?;

    //Return the chatroom if received unique result
    match chatroom.len() {
        1 => Ok(chatroom[0].clone()),
        _ => Err("Chatroom not found".into()),
    }
}

pub async fn get_chatroom_by_id(chatroom_id: u32) -> Result<ChatRoom, Box<dyn Error>> {
    //Connect to database
    let mut conn = get_dbconn().await?.unwrap();

    // Prepare the query
    let query = r"SELECT Id, ChatName, User1_Id, User2_Id FROM chats WHERE Id = :chatroom_id";

    // Execute the query
    let result: Vec<ChatRoom> = conn.exec_map(
        query,
        params! {
            "chatroom_id" => chatroom_id,
        },
        |(id, name, user1_id, user2_id)| ChatRoom::new(id, name, user1_id, user2_id),
    )?;

    //Return the chatroom if received unique result
    match result.len() {
        1 => Ok(result[0].clone()),
        _ => Err("Chatroom not found".into()),
    }
}

pub async fn create_new_friend_request(
    sender_id: u32,
    receiver_id: u32,
) -> Result<(), Box<dyn Error>> {
    //Connect to database
    let mut conn = get_dbconn().await?.unwrap();

    // Prepare the query
    let query = r"INSERT INTO friend_requests (Sender_Id, Receiver_Id, Accepted) VALUES (:sender_id, :receiver_id, 0)";

    // Execute the query
    _ = conn.exec_drop(
        query,
        params! {
            "sender_id" => sender_id,
            "receiver_id" => receiver_id,
        },
    );
    Ok(())
}

pub async fn create_friends(user1_id: u32, user2_id: u32) -> Result<(), Box<dyn Error>> {
    //Connect to database
    let mut conn = get_dbconn().await?.unwrap();

    // Prepare the query
    let query = r"INSERT INTO friends (User1_Id, User2_Id) VALUES (:user1_id, :user2_id)";

    // Execute the query
    _ = conn.exec_drop(
        query,
        params! {
            "user1_id" => user1_id,
            "user2_id" => user2_id,
        },
    );

    Ok(())
}

pub async fn get_friend_requests_by_user_id(
    user_id: u32,
) -> Result<Vec<FriendRequest>, Box<dyn Error>> {
    //Connect to database
    let mut conn = get_dbconn().await?.unwrap();

    // Prepare the query
    let query = r"SELECT fr.Sender_Id, fr.Receiver_Id, fr.Accepted, u.UserName 
                        FROM friend_requests fr
                        INNER JOIN users u ON fr.Sender_Id = u.Id AND fr.Accepted = FALSE AND fr.Receiver_Id = :user_id";

    // Execute the query
    let result: Vec<FriendRequest> = conn.exec_map(
        query,
        params! {
            "user_id" => user_id,
        },
        |(sender_id, receiver_id, is_accepted, display_name)| {
            FriendRequest::new(sender_id, receiver_id, is_accepted, display_name)
        },
    )?;

    Ok(result)
}

pub async fn accept_friend_request_for_user(
    sender_id: u32,
    receiver_id: u32,
) -> Result<(), Box<dyn Error>> {
    //Connect to database
    let mut conn = get_dbconn().await?.unwrap();

    // Prepare the query
    let query = r"UPDATE friend_requests SET Accepted = TRUE WHERE Sender_Id = :sender_id AND Receiver_Id = :receiver_id";

    // Execute the query
    _ = conn.exec_drop(
        query,
        params! {
            "sender_id" => sender_id,
            "receiver_id" => receiver_id,
        },
    );

    Ok(())
}

pub async fn delete_friend_request(sender_id: u32, receiver_id: u32) -> Result<(), Box<dyn Error>> {
    //Connect to database
    let mut conn = get_dbconn().await?.unwrap();

    // Prepare the query
    let query =
        r"DELETE FROM friend_requests WHERE Sender_Id = :sender_id AND Receiver_Id = :receiver_id";

    // Execute the query
    _ = conn.exec_drop(
        query,
        params! {
            "sender_id" => sender_id,
            "receiver_id" => receiver_id,
        },
    );

    Ok(())
}

pub async fn check_if_two_users_are_friends(
    user1_id: u32,
    user2_id: u32,
) -> Result<bool, Box<dyn Error>> {
    //Connect to database
    let mut conn = get_dbconn().await?.unwrap();

    // Prepare the query
    let query = r"SELECT Id FROM friends WHERE (User1_Id = :user1_id AND User2_Id = :user2_id) OR (User1_Id = :user2_id AND User2_Id = :user1_id)";

    // Execute the query
    let result: Option<u32> = conn.exec_first(
        query,
        params! {
            "user1_id" => user1_id,
            "user2_id" => user2_id,
        },
    )?;

    //Return true if received unique result
    Ok(result.is_some())
}

/// Async method to save a chat message to the database
pub async fn save_chat_message_to_database(
    chatroom_id: u32,
    message: String,
) -> Result<(), Box<dyn Error>> {
    //Connect to database
    let mut conn = get_dbconn().await?.unwrap();

    // Prepare the query
    let query = r"INSERT INTO chat_messages (Chat_Id, Message) VALUES (:chatroom_id, :message)";

    // Execute the query
    _ = conn.exec_drop(
        query,
        params! {
            "chatroom_id" => chatroom_id,
            "message" => message,
        },
    );
    Ok(())
}

/// Async method to get chat messages from the database
pub async fn get_chat_messages_for_chatroom_from_database(
    chatroom_id: u32,
) -> Result<Vec<String>, Box<dyn Error>> {
    //Connect to database
    let mut conn = get_dbconn().await?.unwrap();

    // Prepare the query
    let query = r"SELECT Message FROM chat_messages WHERE Chat_Id = :chatroom_id";

    // Execute the query
    let result: Vec<String> = conn.exec_map(
        query,
        params! {
            "chatroom_id" => chatroom_id,
        },
        |message| message,
    )?;

    Ok(result)
}

///Tests for sql_interaction
#[cfg(test)]
mod tests {
    use super::*;
    use crate::login::hash_password;

    #[tokio::test]
    async fn test_check_if_username_exists_positive() {
        let result = check_if_username_exists("rino").await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), true);
    }

    #[tokio::test]
    async fn test_check_if_username_exists_negative() {
        let result = check_if_username_exists("KeinGültigerUsername").await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), false);
    }

    #[tokio::test]
    async fn test_check_if_password_matches_username_positive() {
        let password = "Ifuckingloverust1";
        let result =
            check_if_password_matches_username(hash_password(password.trim()).as_str(), "rino")
                .await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), true);
    }

    #[tokio::test]
    async fn test_check_if_password_matches_username_negative() {
        let result = check_if_password_matches_username("password", "rino").await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), false);
    }

    ///Positive test for save_new_user_to_database_after_signup
    #[tokio::test]
    async fn test_save_new_user_to_database_after_signup() -> Result<(), Box<dyn Error>> {
        //Get database connection
        let mut conn = get_dbconn().await?.unwrap();

        //Testdata
        let username = String::from("testusername");
        let password = String::from("password".to_string());

        //Call save_new_user_to_database_after_signup to test this function
        let result = save_new_user_to_database_after_signup(&username, &password).await;

        //Check if call was successful
        assert!(result.is_ok());

        // Überprüfen, ob der Datensatz korrekt in die Tabelle eingefügt wurde
        let row: Option<(String, String)> = conn.exec_first(
            "SELECT UserName, Password FROM users WHERE UserName = :username",
            params! {
                "username" => username.clone()
            },
        )?;

        assert!(row.is_some());
        let (db_username, db_password) = row.unwrap();
        assert_eq!(db_username, username);
        assert_eq!(db_password, password);

        // Nach dem Test: Tabelle wieder bereinigen
        conn.exec_drop("DELETE FROM users WHERE UserName = 'testusername'", ())?;

        Ok(())
    }

    ///Positive test for check_if_username_exists
    #[tokio::test]
    async fn test_get_username_from_database_positive() {
        //Testdata
        let username: String = String::from("rino");

        //Call check_if_username_exists to test this function
        let result = get_user_from_database_by_name(username.clone()).await;

        //Check if call was successful
        assert!(result.is_ok());
    }

    ///Positive test for check_if_username_exists
    #[tokio::test]
    async fn test_get_username_from_database_negative() {
        //Invalid username
        let username: String = String::from("not_a_username");

        //Call check_if_username_exists to test this function
        let result = get_user_from_database_by_name(username.clone()).await;

        //Check result
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_check_if_user_isblocked_positive() {
        let username: String = String::from("rino");

        _ = set_user_isblocked(username.clone(), 35).await;

        let result = check_if_user_isblocked(username.clone()).await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), true);
    }

    #[tokio::test]
    async fn test_check_if_user_isblocked_negative() {
        let username: String = String::from("anton");

        let result = check_if_user_isblocked(username.clone()).await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), false);
    }

    #[tokio::test]
    async fn test_check_if_user_isblocked_user_not_found() {
        //Testdata
        let username: String = String::from("KeinGültigerUsername");

        //Call check_if_username_exists to test this function
        let result = check_if_user_isblocked(username.clone()).await;

        //Check if call was successful
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), false);
    }

    #[tokio::test]
    async fn test_set_user_is_blocked_positive() {
        let username: String = String::from("rino");

        _ = set_user_isblocked(username.clone(), 5);

        let result = get_user_is_blockeduntil(username.clone()).await;

        assert!(result.is_ok());
        assert!(result.unwrap() > helper_functions::get_sys_time_in_secs());
    }

    ///Positive test for get_user_by_name_with_contains_search
    #[tokio::test]
    async fn test_get_user_by_name_with_contains_search_positive() {
        //Testdata
        let search_input: String = String::from("anton");
        let user_id = 2; // Id for rino

        let result = get_user_by_name_with_contains_search(search_input, user_id).await;

        //Check if call was successful
        assert!(result.is_ok());

        // Unwrap the result to get the vector
        let users = result.unwrap();

        // Check if the resulting vector has exactly two elements (antonius and antonia, anton is already friend of rino, so he should not be in the result)
        assert_eq!(users.len(), 2, "Expected exactly two users in the result");

        // Check if the names are antonius and antonia
        let user_names: Vec<&str> = users.iter().map(|user| user.get_name().as_str()).collect();
        assert!(
            user_names.contains(&"antonius"),
            "Expected user 'antonius' to be in the result"
        );
        assert!(
            user_names.contains(&"antonia"),
            "Expected user 'antonia' to be in the result"
        );
    }

    ///Positive test for get_user_by_name_with_contains_search
    #[tokio::test]
    async fn test_get_user_by_name_with_contains_search_negative() {
        //Testdata
        let search_input: String = String::from("no_valid_username");
        let user_id = 2; // Id for rino

        let result = get_user_by_name_with_contains_search(search_input, user_id).await;

        //Check if call was successful
        assert!(result.is_ok());

        // Unwrap the result to get the vector
        let users = result.unwrap();

        // Check if the resulting vector has exactly two elements
        assert_eq!(users.len(), 0, "Expected exactly zero users in the result");
    }

    /// Test for getting friend requests by user id
    #[tokio::test]
    async fn test_get_friend_request_for_user() {
        let user_id: u32 = 3;

        let result = get_friend_requests_by_user_id(user_id).await;

        // user 3 should have 2 friend request that are not accepted
        assert_eq!(result.unwrap().len(), 2);
    }

    /// Test for declining friend requests
    #[tokio::test]
    async fn test_declining_friend_request() {
        //Invalid username
        let username: String = String::from("not_a_username");

        let result = get_user_from_database_by_name(username.clone()).await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_create_friend_request() {
        let sender_id: u32 = 1;
        let receiver_id: u32 = 2;

        let result = create_new_friend_request(sender_id, receiver_id).await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_check_if_two_users_are_friends_positive() {
        let user1_id: u32 = 1;
        let user2_id: u32 = 2;

        let result = check_if_two_users_are_friends(user1_id, user2_id).await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), true);
    }

    #[tokio::test]
    async fn test_check_if_two_users_are_friends_negative() {
        let user1_id: u32 = 1;
        let user2_id: u32 = 4;

        let result = check_if_two_users_are_friends(user1_id, user2_id).await;

        //Check result
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), false);
    }

    #[tokio::test]
    async fn test_get_chatroom_by_id_positive() {
        let chatroom_id: u32 = 1;

        let result = get_chatroom_by_id(chatroom_id).await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_get_chatroom_by_id_negative() {
        let chatroom_id: u32 = 999999;

        let result = get_chatroom_by_id(chatroom_id).await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_save_message_to_database() {
        let chatroom_id: u32 = 1;
        let message: String = String::from("Testmessage");

        let result = save_chat_message_to_database(chatroom_id, message).await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_get_chat_messages_for_chatroom_from_database() {
        let chatroom_id: u32 = 1;

        let result = get_chat_messages_for_chatroom_from_database(chatroom_id).await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_create_friends() {
        let user1_id: u32 = 4;
        let user2_id: u32 = 5;

        let result = create_friends(user1_id, user2_id).await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_decline_friend_request() {
        let sender_id: u32 = 2;
        let receiver_id: u32 = 4;

        let result = delete_friend_request(sender_id, receiver_id).await;

        assert!(result.is_ok());
    }
}
