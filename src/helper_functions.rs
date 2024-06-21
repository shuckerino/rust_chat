use colored::*;
use dialoguer::theme::ColorfulTheme;
use dialoguer::Select;
use std::process::Command;
use std::time::SystemTime;

// Method for getting the current timestamp in seconds since UNIX EPOCH
pub fn get_sys_time_in_secs() -> u64 {
    match SystemTime::now().duration_since(SystemTime::UNIX_EPOCH) {
        Ok(n) => n.as_secs(),
        Err(_) => panic!("SystemTime before UNIX EPOCH!"),
    }
}

/// Method for clearing the console
pub fn clear_console() -> Result<(), Box<dyn std::error::Error>> {
    #[cfg(target_os = "windows")]
    let _ = Command::new("cmd").arg("/c").arg("cls").status();

    #[cfg(not(target_os = "windows"))]
    let _ = Command::new("sh").arg("-c").arg("clear").status();

    Ok(())
}

/// Method for reading user input from the console
pub fn read_user_input_from_console() -> String {
    let mut input = String::new();
    std::io::stdin().read_line(&mut input).unwrap();
    input.trim().to_string()
}

/// Method for displaying a list of choices to the user
pub fn display_multiple_choices(
    prompt: &str,
    list_of_choices: Vec<String>,
    with_clear_console: bool,
) -> usize {
    if with_clear_console {
        _ = clear_console();
    }
    let selection = Select::with_theme(&ColorfulTheme::default())
        .with_prompt(prompt)
        .default(0)
        .items(&list_of_choices)
        .interact()
        .unwrap();
    selection
}

/// Method for asking the user a yes/no question
/// <param name="question">The question to ask the user</param>
/// <returns>True if the user answers "Yes", False if the user answers "No"</returns>
pub fn ask_user_yes_no_question(question: &str) -> bool {
    let choices = vec!["Yes".to_string(), "No".to_string()];
    let selection = display_multiple_choices(question, choices, true);
    selection == 0
}

/// Method to print an info message to the console in yellow
pub fn print_info(msg: &str) -> Result<(), Box<dyn std::error::Error>> {
    println!("{}", msg.yellow());
    Ok(())
}

/// Method to print an error message to the console in red
pub fn print_error(msg: &str) -> Result<(), Box<dyn std::error::Error>> {
    println!("{}", msg.red());
    Ok(())
}

/// Method to print an confirmation message to the console in green
pub fn print_confirmation(msg: &str) -> Result<(), Box<dyn std::error::Error>> {
    println!("{}", msg.green());
    Ok(())
}

///Tests for login
#[cfg(test)]
mod tests {
    use super::*;
    use std::thread::sleep;
    use std::time::Duration;

    #[test]
    fn test_print_info() {
        let status = print_info("This is an info message");
        assert!(status.is_ok());
    }

    #[test]
    fn test_print_error() {
        let status = print_error("This is an error message");
        assert!(status.is_ok());
    }
    #[test]
    fn test_print_confirmation() {
        let status = print_confirmation("This is a confirmation message");
        assert!(status.is_ok());
    }

    #[test]
    fn test_clear_console() {
        let status = clear_console();
        assert!(status.is_ok());
    }

    ///Test for get_sys_time_in_secs
    #[test]
    fn test_get_sys_time_in_secs() {
        let time1 = get_sys_time_in_secs();

        sleep(Duration::from_secs(1));

        let time2 = get_sys_time_in_secs();

        // Check if time2 is time 1 plus 1 since we waited for one second
        assert_eq!(time2, time1 + 1);
    }
}
