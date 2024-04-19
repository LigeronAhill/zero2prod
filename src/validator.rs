use unicode_segmentation::UnicodeSegmentation;
pub fn validate_name(name: &str) -> bool {
    !(contains_forbidden_chars(name) || is_too_long(name) || is_empty_or_whitespace(name))
}
fn is_too_long(value: &str) -> bool {
    value.graphemes(true).count() > 256
}
fn is_empty_or_whitespace(value: &str) -> bool {
    value.trim().is_empty()
}
fn contains_forbidden_chars(value: &str) -> bool {
    let forbidden_characters = ['/', '(', ')', '"', '<', '>', '\\', '{', '}'];
    value.chars().any(|c| forbidden_characters.contains(&c))
}

#[cfg(test)]
mod tests {
    use crate::validator::validate_name;

    #[test]
    fn too_long_name() {
        let name = "Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod tempor incididunt ut labore et dolore magna aliqua. Ut enim ad minim veniam, quis nostrud exercitation ullamco laboris nisi ut aliquip ex ea commodo consequat. Duis aute irure dolor in reprehenderit in voluptate velit esse cillum dolore eu fugiat nulla pariatur. Excepteur sint occaecat cupidatat non proident, sunt in culpa qui officia deserunt mollit anim id est laborum.";
        assert!(!validate_name(name))
    }
    #[test]
    fn empty_name() {
        let name = "";
        assert!(!validate_name(name))
    }
    #[test]
    fn whitespace_name() {
        let name = " ";
        assert!(!validate_name(name))
    }
    #[test]
    fn forbidden_characters() {
        let name = "<h1>Hello</h1>";
        assert!(!validate_name(name))
    }
    #[test]
    fn right_name() {
        let name = "Ursule LeGuin";
        assert!(validate_name(name))
    }
}
