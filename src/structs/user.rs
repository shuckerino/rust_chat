#[derive(Debug, PartialEq, Clone, Eq)]
pub struct User {
    id: u32,
    name: String,
}

impl User {
    // Constructor
    pub fn new(id: u32, name: String) -> User {
        User { id, name }
    }

    pub fn get_name(&self) -> &String {
        &self.name
    }

    pub fn get_id(&self) -> u32 {
        self.id
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_user_new() {
        let user = User::new(1, "Alice".to_string());
        assert_eq!(user.id, 1);
        assert_eq!(user.name, "Alice".to_string());
    }

    #[test]
    fn test_get_user_name() {
        let user = User::new(1, "Alice".to_string());
        assert_eq!(*user.get_name(), "Alice".to_string());
    }

    #[test]
    fn test_get_user_id() {
        let user = User::new(1, "Alice".to_string());
        assert_eq!(user.get_id(), 1);
    }
}
