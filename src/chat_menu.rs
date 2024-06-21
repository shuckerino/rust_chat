use std::{thread::sleep, time::Duration};

use crate::{
    helper_functions, sql_interaction,
    structs::{chat_room::ChatRoom, friend_request::FriendRequest, user::User},
};

/// Main loop for the chat menu
/// <br>Returns the selected chatroom if the user selected one, None otherwise
pub async fn chat_menu_loop(current_user: User) -> Option<ChatRoom> {
    loop {
        let welcome_prompt = format!("Hello {}, what do you want to do?", current_user.get_name());
        let list_of_choices = vec![
            "Search for friend".to_string(),
            "Join existing chatroom".to_string(),
            "Check friend requests".to_string(),
            "Log out".to_string(),
        ];
        let selection =
            helper_functions::display_multiple_choices(&welcome_prompt, list_of_choices, true);

        match selection {
            0..=2 => {
                if let Some(chat_room) = chat_menu_selection(selection, current_user.clone()).await
                {
                    return Some(chat_room);
                }
            }
            3 => {
                //user chose third option - Exit
                _ = helper_functions::print_info("Logging out...");
                sleep(Duration::from_secs(1));
                return None;
            }
            _ => unreachable!("Invalid selection in chat menu!"),
        }
    }
}

/// Method for handling the user selection in the chat menu
async fn chat_menu_selection(selection: usize, current_user: User) -> Option<ChatRoom> {
    match selection {
        0 => {
            _ = helper_functions::clear_console();
            // User chose to search for new friend
            _ = helper_functions::print_info(
                "Please enter a username you want to search for (leave blank to return):",
            );
            let search_input = helper_functions::read_user_input_from_console();

            // Return to chat menu
            if search_input.is_empty() {
                return None;
            }

            let prompt = format!(
                "Please wait while we search for users containing \"{}\"...",
                search_input
            );
            _ = helper_functions::print_info(&prompt);

            let found_users = sql_interaction::get_user_by_name_with_contains_search(
                search_input,
                current_user.get_id(),
            )
            .await;
            match found_users {
                Ok(users) => {
                    if users.is_empty() {
                        _ = helper_functions::print_error(
                            "No users found with that username. Please try again.",
                        );
                        sleep(Duration::from_secs(2));
                        None
                    } else {
                        let selected_user = select_friend_for_list_of_users(users.clone());
                        match selected_user {
                            Some(user) => {
                                let is_already_friends =
                                    sql_interaction::check_if_two_users_are_friends(
                                        current_user.get_id(),
                                        user.get_id(),
                                    )
                                    .await
                                    .unwrap();

                                if is_already_friends {
                                    let msg = format!("You are already friends with {}. Please select another user.", user.get_name());
                                    _ = helper_functions::print_info(&msg);
                                    sleep(Duration::from_secs(2));
                                    None
                                } else {
                                    let msg = format!("Sending friend request to {}. If the request gets accepted, you will see the new chat as an existing chatroom!", user.get_name());
                                    _ = helper_functions::print_confirmation(&msg);
                                    _ = sql_interaction::create_new_friend_request(
                                        current_user.get_id(),
                                        user.get_id(),
                                    )
                                    .await;
                                    sleep(Duration::from_secs(2));
                                    None
                                }
                            }
                            None => None,
                        }
                    }
                }
                Err(e) => {
                    eprintln!("Error searching for users: {}", e);
                    None
                }
            }
        }
        1 => {
            // User chose to join an existing chatroom

            joining_an_existing_chatroom(current_user.clone()).await
        }
        2 => {
            // User chose to see friend requests
            _ = helper_functions::print_info(
                "Please wait while we search for your friend requests...",
            );
            // Get all friend requests for the current user
            let friend_requests =
                sql_interaction::get_friend_requests_by_user_id(current_user.get_id()).await;

            match friend_requests {
                Ok(requests) => {
                    if requests.is_empty() {
                        _ = helper_functions::print_confirmation(
                            "Currently, you do not have any friend requests!",
                        );
                        sleep(std::time::Duration::from_secs(2));
                        None
                    } else {
                        let selected_friend_request =
                            list_all_friend_requests(current_user.clone(), requests);

                        match selected_friend_request {
                            Some(friend_request) => {
                                let should_accept =
                                    helper_functions::ask_user_yes_no_question(&format!(
                                        "Do you want to accept the friend request from {}?",
                                        friend_request.get_display_name()
                                    ));

                                if should_accept {
                                    friend_request
                                        .accept_friend_request(current_user.clone())
                                        .await
                                } else {
                                    let _ = friend_request
                                        .decline_friend_request(current_user.get_id())
                                        .await;
                                    None
                                }
                            }
                            None => None,
                        }
                    }
                }
                Err(e) => {
                    eprintln!("Error searching for friend requests: {}", e);
                    None
                }
            }
        }
        _ => unreachable!(),
    }
}

/// Method to search for a friend by username
/// <br>Returns the user if found, None otherwise
fn select_friend_for_list_of_users(list_of_users: Vec<User>) -> Option<User> {
    let mut user_choices = list_of_users
        .iter()
        .map(|user| user.get_name().clone())
        .collect::<Vec<String>>();
    user_choices.push("Go back".to_string());
    let selection = helper_functions::display_multiple_choices(
        "Select a user to send a friend request.",
        user_choices,
        true,
    );
    get_user_by_selection(list_of_users, selection)
}

fn get_user_by_selection(users: Vec<User>, selection: usize) -> Option<User> {
    let index = selection;

    if index == users.len() {
        None
    } else {
        Some(users[selection].clone())
    }
}

/// Method for listing all previous chatrooms for the current user and asking the user to select one
async fn joining_an_existing_chatroom(current_user: User) -> Option<ChatRoom> {
    _ = helper_functions::print_info("Please wait while we search for your chatrooms...");
    let user_chatrooms = sql_interaction::search_for_chatrooms_of_user(current_user.get_id()).await;
    _ = helper_functions::clear_console();

    match user_chatrooms {
        Ok(chatrooms) => {
            if chatrooms.is_empty() {
                _ = helper_functions::print_info(
                    "You do not have any chatrooms yet! Try to search for new friends!",
                );
                sleep(Duration::from_secs(2));
                None
            } else {
                let mut list_of_choices = chatrooms
                    .iter()
                    .map(|chatroom| chatroom.get_name().clone())
                    .collect::<Vec<String>>();
                // list_of_choices.push("Refresh".to_string());
                list_of_choices.push("Go back".to_string());
                let selection = helper_functions::display_multiple_choices(
                    "Select a chatroom to connect to:",
                    list_of_choices,
                    true,
                );

                // return the selected chatroom
                joining_existing_chatroom_selection(selection, chatrooms.clone()).await
            }
        }
        Err(e) => {
            eprintln!("Error searching for chatrooms: {}", e);
            None
        }
    }
}

async fn joining_existing_chatroom_selection(
    selected_index: usize,
    chatrooms: Vec<ChatRoom>,
) -> Option<ChatRoom> {
    let index = selected_index;
    if index == chatrooms.len() {
        //user chose to go back to main menu
        None
    } else {
        //user chose a chatroom
        Some(chatrooms[index].clone())
    }
}

fn list_all_friend_requests(
    _current_user: User,
    friend_requests: Vec<FriendRequest>,
) -> Option<FriendRequest> {
    // Let the user select a friend request to accept or reject
    let mut list_of_choices = friend_requests
        .iter()
        .map(|request| request.get_display_name().clone())
        .collect::<Vec<String>>();
    list_of_choices.push("Refresh".to_string());
    list_of_choices.push("Go to main menu".to_string());
    let selection = helper_functions::display_multiple_choices(
        "Select a friend request to accept or reject:",
        list_of_choices,
        true,
    );

    let index = selection;

    if index == friend_requests.len() + 1 {
        //user chose to go back to main menu
        None
    } else if index == friend_requests.len() {
        //user chose to refresh the friend requests
        list_all_friend_requests(_current_user, friend_requests)
    } else {
        //user chose a friend request
        Some(friend_requests[index].clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Test if user selected a chatroom in the "List all previous chats" menu
    #[tokio::test]
    async fn test_previous_chat_selection_chatroom_got_selected() {
        let dummy_chatrooms = vec![
            ChatRoom::new(1, "Chatroom 1".to_string(), 1, 2),
            ChatRoom::new(2, "Chatroom 2".to_string(), 2, 3),
        ];
        let res = joining_existing_chatroom_selection(0, dummy_chatrooms).await;

        assert!(res.is_some());
        assert_eq!(*res.unwrap().get_name(), "Chatroom 1".to_string());
    }

    /// Test if user selected to go back to the chat menu in the "List all previous chats" menu
    #[tokio::test]
    async fn test_previous_chat_selection_go_to_chat_menu() {
        let dummy_chatrooms = vec![
            ChatRoom::new(1, "Chatroom 1".to_string(), 1, 2),
            ChatRoom::new(2, "Chatroom 2".to_string(), 2, 3),
        ];
        let res = joining_existing_chatroom_selection(2, dummy_chatrooms).await;
        assert!(res.is_none());
    }

    #[test]
    fn test_get_user_by_selection_positive() {
        let users = vec![
            User::new(1, "Alice".to_string()),
            User::new(2, "Bob".to_string()),
        ];
        let result = get_user_by_selection(users.clone(), 1);
        assert_eq!(result, Some(users[1].clone()));
    }

    #[test]
    fn test_get_user_by_selection_exit() {
        let users = vec![
            User::new(1, "Alice".to_string()),
            User::new(2, "Bob".to_string()),
        ];
        let result = get_user_by_selection(users, 2);
        assert_eq!(result, None);
    }

    #[test]
    #[should_panic]
    fn test_out_of_bounds_selection() {
        let users = vec![
            User::new(1, "Alice".to_string()),
            User::new(2, "Bob".to_string()),
        ];
        // This should panic because it's out of bounds
        get_user_by_selection(users, 3);
    }

    #[tokio::test]
    async fn test_joining_chatroom_with_user_having_no_existing_chatrooms() {
        let user = User::new(4, "antonia".to_string());
        let res = joining_an_existing_chatroom(user.clone()).await;
        assert!(res.is_none());
    }
}
