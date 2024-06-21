use crate::helper_functions;
use crate::sql_interaction;
use crate::structs::user::User;
use colored::Colorize;
use regex::Regex;
use rpassword::read_password;
use sha2::{Digest, Sha256};
use std::fmt::Write;
use std::io::{self};
use std::thread;
use std::time::Duration;

extern crate bcrypt;
extern crate sha2;

///Start the authentication process when a client starts the application
pub async fn start_authentication_process_for_client() -> Option<User> {
    _ = helper_functions::clear_console();

    println!(
        "{}",
        "Welcome to our ChatClient!".bold().purple()
    );

    let choices = vec![
        "Login".to_string(),
        "Sign up".to_string(),
        "Exit".to_string(),
    ];
    let selection = helper_functions::display_multiple_choices("Choose an option", choices, false);

    match selection {
        0 => {
            // User chose login
            _ = helper_functions::clear_console();
            // return current user
            user_chose_login().await
        }
        1 => {
            // User chose sign up
            _ = helper_functions::clear_console();
            // return current user
            user_chose_signup().await
        }
        2 => {
            // User chose Exit
            _ = helper_functions::clear_console();
            println!("{}", "Goodbye, we hope to see you again!".bold().purple());
            std::process::exit(0);
        }
        _ => unreachable!(), //Impossible since only 0, 1 and 2 can be selected
    }
}

///Hashes a string<br/>
///Used to hash passwords that can be stored in database
pub fn hash_password(password: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(password);
    let result = hasher.finalize();
    result.iter().fold(String::new(), |mut acc, byte| {
        write!(acc, "{:02x}", byte).expect("Unable to write");
        acc
    })
}

///Lets the user login
async fn user_chose_login() -> Option<User> {
    let mut username: String;
    loop {
        //Get valid username from user input
        username = enter_username_for_login().await;

        if username.is_empty() {
            //user wants to return
            return None;
        } else {
            //Check if user is blocked
            if sql_interaction::check_if_user_isblocked(username.clone())
                .await
                .unwrap()
            {
                for second in (1..=5).rev() {
                    _ = helper_functions::clear_console();
                    let mut msg = format!(
                    "Account {} is currently blocked due to too many failed attempts. Try again later.",
                    username
                );
                    println!("{}", msg.bold().red());
                    msg = format!("Returning to main menu in {}...", second);
                    println!("{}", msg.bold());
                    thread::sleep(Duration::from_secs(1));
                }

                //Return to main menu
                return None;
            } else if !enter_password_for_login(username.clone()).await
                && !sql_interaction::check_if_user_isblocked(username.clone())
                    .await
                    .unwrap()
            {
                _ = helper_functions::clear_console();
                continue;
            } else {
                // TODO: Error handling
                let user: User = sql_interaction::get_user_from_database_by_name(username.clone())
                    .await
                    .unwrap();

                return Some(user);
            }
        }
    }
}

///Allows the user to create a new account
async fn user_chose_signup() -> Option<User> {
    // Get valid username from user input
    let username: String = enter_username_for_signup().await?;

    let new_password = enter_new_password_for_signup()?;
    let new_password_str: String = new_password.trim().to_string();

    if repeat_password_for_signup(&new_password_str).is_some() {
        sql_interaction::save_new_user_to_database_after_signup(
            &username,
            &hash_password(&new_password_str),
        )
        .await
        .expect("TODO: Could not save user to database");

        let user: User = sql_interaction::get_user_from_database_by_name(username.clone())
            .await
            .unwrap();

        _ = helper_functions::print_confirmation("Account successfully created!");
        thread::sleep(Duration::from_secs(2));

        Some(user)
    } else {
        // User wants to go back
        None
    }
}

///Ask user to enter a username until user entered a valid username on login
async fn enter_username_for_login() -> String {
    let mut username;

    //Enter new username until username is valid --> check if user exists in db to avoid SQL-Injection
    loop {
        _ = helper_functions::print_info("Enter username: (leave blank to return)");
        username = String::new();
        match io::stdin().read_line(&mut username) {
            Ok(_) => {}
            Err(_err) => {
                _ = helper_functions::print_error("Error while reading username\n");
                continue;
            }
        }

        username = username.trim().to_string();

        //return if user can not login
        if username.is_empty() {
            break;
        }

        let username_does_exist = sql_interaction::check_if_username_exists(username.trim())
            .await
            .unwrap();

        if !username_does_exist {
            _ = helper_functions::print_error("Unknown username!\n");
            thread::sleep(Duration::from_secs(2));
            _ = helper_functions::clear_console();
            continue;
        }

        if is_corresponding_to_the_username_regulations(username.trim())
            && is_avoiding_sql_injection(username.trim())
        {
            _ = helper_functions::clear_console();
            break;
        }
    }

    username
}

///Ask user to enter the password for the account with the username to login
async fn enter_password_for_login(username: String) -> bool {
    let mut tries_for_password = 0;

    loop {
        _ = helper_functions::clear_console();

        if tries_for_password > 0 {
            let msg = format!(
                "Password is incorrect (Tries left: {})",
                3 - tries_for_password
            );
            _ = helper_functions::print_error(&msg);
        }

        _ = helper_functions::print_info("Please enter your password: (leave blank to return)");
        let password = read_password().expect("Error while reading password");
        // let password = "Ifuckingloverust1";

        if password.trim() == "" {
            return false;
        }

        if is_corresponding_to_the_password_regulations(&password)
            && is_avoiding_sql_injection(&password)
            && sql_interaction::check_if_password_matches_username(
                hash_password(password.trim()).as_str(),
                username.as_str(),
            )
            .await
            .unwrap()
        {
            return true;
        }

        tries_for_password += 1;

        if tries_for_password == 3 {
            //Block user -> refresh IsBlocked in database
            sql_interaction::set_user_isblocked(username.clone(), 20)
                .await
                .expect("Could not update IsBlocked value");

            //User needed too many tries --> block account and return to main menu
            for second in (1..=5).rev().step_by(2) {
                let remaining_seconds = sql_interaction::get_user_is_blockeduntil(username.clone())
                    .await
                    .unwrap()
                    - helper_functions::get_sys_time_in_secs();
                _ = helper_functions::clear_console();
                let mut msg = format!(
                    "Account {} was blocked for {} more seconds due to too many failed attempts",
                    username, remaining_seconds
                );
                println!("{}", msg.bold().red());
                msg = format!("Returning to main menu in {}...", second);
                println!("{}", msg.bold());
                thread::sleep(Duration::from_secs(1));
            }

            return false;
        }
    }
}

///Ask user to enter a username to create a new account on signup
async fn enter_username_for_signup() -> Option<String> {
    _ = helper_functions::clear_console();
    let mut username;

    //Enter new username until username is valid --> check if user exists in db to avoid SQL-Injection
    loop {
        //Ask user to enter username
        _ = helper_functions::print_info("Enter your desired username: (leave blank to return)");
        username = String::new();
        io::stdin()
            .read_line(&mut username)
            .expect("Failed to read username"); // Read user input

        username = username.trim().to_string();

        //return if user can not login
        if username.is_empty() {
            return None;
        }

        if !is_corresponding_to_the_username_regulations(username.trim()) {
            let error_message = r#"
            Invalid username!
            - at least 3 characters
            - only alphanumeric characters are allowed
            "#;
            _ = helper_functions::print_error(error_message);
            thread::sleep(Duration::from_secs(2));
            continue;
        } else if !is_avoiding_sql_injection(username.trim()) {
            let error_message = "Your name contains invalid characters. Please try again.";
            _ = helper_functions::print_error(error_message);
            thread::sleep(Duration::from_secs(2));
            continue;
        } else if sql_interaction::check_if_username_exists(username.trim())
            .await
            .unwrap()
        {
            let error_message = "This username is already taken. Please try again.";
            _ = helper_functions::print_error(error_message);
            thread::sleep(Duration::from_secs(2));
            continue;
        }

        _ = helper_functions::clear_console();
        return Some(username);
    }
}

///Ask user to enter a password for the new account on signup
fn enter_new_password_for_signup() -> Option<String> {
    //Ask user to enter a password
    loop {
        _ = helper_functions::clear_console();
        _ = helper_functions::print_info("Please enter your password: (leave blank to return)");
        let new_password = read_password().expect("Fehler beim Lesen des Passworts");
        if new_password.trim().is_empty() {
            _ = helper_functions::clear_console();
            return None;
        }

        if !is_avoiding_sql_injection(&new_password) {
            _ = helper_functions::print_error(
                "There are invalid characters in the password. Please try again.",
            );
            thread::sleep(Duration::from_secs(3));
            continue;
        }

        if !is_corresponding_to_the_password_regulations(&new_password) {
            _ = helper_functions::print_error("The password does not meet the security requirements. Make sure it is at least 12 characters, has upper and lower case and at least one digt.");
            thread::sleep(Duration::from_secs(3));
            continue;
        }

        return Some(new_password);
    }
}

///Ask user to repeat the password on signup
fn repeat_password_for_signup(password: &str) -> Option<String> {
    //Ask user to repeat the password
    loop {
        _ = helper_functions::clear_console();

        _ = helper_functions::print_info("Please repeat your password: (leave blank to return)");
        let new_password = read_password().expect("Fehler beim Lesen des Passworts");

        if new_password.trim() == "" {
            _ = helper_functions::clear_console();
            return None;
        }

        if password.trim() != new_password.trim() {
            _ = helper_functions::print_error("Passwords do not match. Please try again.");
            thread::sleep(Duration::from_secs(2));
            continue;
        }

        if !is_avoiding_sql_injection(&new_password) {
            _ = helper_functions::print_error(
                "There are invalid characters in the password. Please try again.",
            );
            thread::sleep(Duration::from_secs(2));
            continue;
        }

        if !is_corresponding_to_the_password_regulations(&new_password) {
            _ = helper_functions::print_error("The password does not meet the security requirements. Make sure it is at least 12 characters, has upper and lower case and at least one digt.");
            thread::sleep(Duration::from_secs(2));
            continue;
        }

        // Successfully passed all tests again
        _ = helper_functions::print_confirmation("Password is valid");
        _ = helper_functions::clear_console();
        return Some(password.to_string());
    }
}

///Checks if a string (username) meets all username criterias
fn is_corresponding_to_the_username_regulations(username: &str) -> bool {
    //Check if username is long enough
    if username.len() < 3 {
        return false;
    }

    //Check if all characters are alphanumeric
    for c in username.chars() {
        if !c.is_alphanumeric() {
            return false; //username not valid if a char is not alphanumeric
        }
    }

    true
}

///Checks if a string (password) meets all password criterias
fn is_corresponding_to_the_password_regulations(password: &str) -> bool {
    // - Mindestens 12 Zeichen lang
    // - Mindestens ein Großbuchstabe
    // - Mindestens ein Kleinbuchstabe
    // - Mindestens eine Zahl
    let length_check = password.len() >= 12;
    let uppercase_check = Regex::new(r"[A-Z]").unwrap().is_match(password);
    let lowercase_check = Regex::new(r"[a-z]").unwrap().is_match(password);
    let digit_check = Regex::new(r"\d").unwrap().is_match(password);

    length_check && uppercase_check && lowercase_check && digit_check
}

///Checks if a string avoids SQL-injection
fn is_avoiding_sql_injection(text: &str) -> bool {
    // SQL-Injection-Prüfung:
    // Verbotene Zeichen und Sequenzen, die typischerweise in SQL-Injections vorkommen:
    let forbidden_patterns = [
        r"[']",  // Einfache Anführungszeichen
        r"[\\]", // Backslashes
        r"[;]",  // Semikolons
        "\"",    // Doppelte Anführungszeichen
        r"--",   // SQL Kommentar
        r"/\*",  // SQL Kommentar Start
        r"\*/",  // SQL Kommentar Ende
                 //r"\b(SELECT|INSERT|UPDATE|DELETE|DROP|ALTER|CREATE|EXEC)\b", // SQL Schlüsselwörter
    ];

    for pattern in &forbidden_patterns {
        if Regex::new(pattern).unwrap().is_match(text) {
            return false;
        }
    }

    true
}

///Tests for login
#[cfg(test)]
mod tests {
    use super::*;

    ///Positive test for is_corresponding_to_the_username_regulations
    #[test]
    fn test_positive_is_corresponding_to_the_username_regulations() {
        // Positive Test (Meets Username Regulations)
        assert!(is_corresponding_to_the_username_regulations("user123"));
        assert!(is_corresponding_to_the_username_regulations("user123NAME"));
        assert!(is_corresponding_to_the_username_regulations("123username"));
    }

    ///Negative test for is_corresponding_to_the_username_regulations
    #[test]
    fn test_negative_is_corresponding_to_the_username_regulations() {
        // Negative Test (Does Not Meet Username Regulations)
        assert!(!is_corresponding_to_the_username_regulations("u"));
        assert!(!is_corresponding_to_the_username_regulations("us"));
        assert!(!is_corresponding_to_the_username_regulations("us!"));
        assert!(!is_corresponding_to_the_username_regulations("user name"));
    }

    ///Positive test for is_corresponding_to_the_password_regulations
    #[test]
    fn test_positive_is_corresponding_to_the_password_regulations() {
        // Positive Test (Meets Password Regulations)
        assert!(is_corresponding_to_the_password_regulations(
            "StrongPassword123"
        ));
        assert!(is_corresponding_to_the_password_regulations(
            "P@ssw0rdExample"
        ));
        assert!(is_corresponding_to_the_password_regulations(
            "SecureP@55w0rd!"
        ));
    }

    ///Negative test for is_corresponding_to_the_password_regulations
    #[test]
    fn test_negative_is_corresponding_to_the_password_regulations() {
        // Negative Test (Does Not Meet Password Regulations)
        assert!(!is_corresponding_to_the_password_regulations(
            "weakpassword"
        ));
        assert!(!is_corresponding_to_the_password_regulations("1234567890"));
        assert!(!is_corresponding_to_the_password_regulations("TooShort1"));
        assert!(!is_corresponding_to_the_password_regulations(
            "NoDigitsOrUpperCase"
        ));
    }

    ///Positive test for is_avoiding_sql_injection
    #[test]
    fn test_positive_is_avoiding_sql_injection() {
        // Positive Test (No SQL Injection Pattern)
        assert!(is_avoiding_sql_injection("This is a safe string."));
        assert!(is_avoiding_sql_injection("Some other safe input."));
        assert!(is_avoiding_sql_injection("123456"));
        assert!(is_avoiding_sql_injection("Hello, world!"));
    }

    ///Negative test for is_avoiding_sql_injection
    #[test]
    fn test_negative_is_avoiding_sql_injection() {
        // Negative Test (SQL Injection Pattern Detected)
        assert!(!is_avoiding_sql_injection("SELECT * FROM users;"));
        assert!(!is_avoiding_sql_injection("DROP TABLE users;"));
        assert!(!is_avoiding_sql_injection("'; DROP TABLE users; --"));
        assert!(!is_avoiding_sql_injection(
            "INSERT INTO users (name) VALUES ('John');"
        ));
    }

    ///Positive test for hash_password
    #[test]
    fn test_positive_hash_password() {
        // Positive Test (Valid Hash Generation)
        let password1 = "Ifuckingloverust1";
        let password2 = "Ifuckingloverust1";
        let password3 = "Ifuckingloverust2";

        let hashed_password1 = hash_password(password1);
        let hashed_password2 = hash_password(password2);
        let hashed_password3 = hash_password(password3);

        assert_eq!(
            hashed_password1,
            "dc3e5b292a2a16dc1e28f718b96d3096a1e04c6464a1594edd42824eea771766"
        );
        assert_eq!(
            hashed_password2,
            "dc3e5b292a2a16dc1e28f718b96d3096a1e04c6464a1594edd42824eea771766"
        );
        assert_eq!(hashed_password1, hashed_password2);
        assert_ne!(hashed_password1, hashed_password3);
    }

    ///Negative test for hash_password
    #[test]
    fn test_negative_hash_password() {
        // Negative Test (Empty Password)
        let empty_password = "";
        let hashed_empty_password = hash_password(empty_password);
        assert_eq!(
            hashed_empty_password,
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        );
    }
}
