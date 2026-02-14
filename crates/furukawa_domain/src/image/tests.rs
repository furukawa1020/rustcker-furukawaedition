#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_digest() {
        let valid = "sha256:e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855";
        let d = Digest::new(valid);
        assert!(d.is_ok());
        assert_eq!(d.unwrap().as_str(), valid);
    }

    #[test]
    fn test_invalid_digest_prefix() {
        let invalid = "md5:e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855";
        let d = Digest::new(invalid);
        assert!(d.is_err());
    }

    #[test]
    fn test_invalid_digest_length() {
        let invalid = "sha256:short";
        let d = Digest::new(invalid);
        assert!(d.is_err());
    }
    
    #[test]
    fn test_invalid_digest_chars() {
        let invalid = "sha256:g3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"; // 'g' is not hex
        let d = Digest::new(invalid);
        assert!(d.is_err());
    }
}
