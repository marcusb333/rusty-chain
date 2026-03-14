use sha2::{Digest, Sha256};

pub fn hash_sha256(data: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(data);
    format!("{:x}", hasher.finalize())
}

pub fn hash_object<T: serde::Serialize>(obj: &T) -> Result<String, serde_json::Error> {
    let json = serde_json::to_string(obj)?;
    Ok(hash_sha256(json.as_bytes()))
}

pub fn check_pow(hash: &str, difficulty: u32) -> bool {
    hash.starts_with(&"0".repeat(difficulty as usize))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hash_sha256() {
        let hash1 = hash_sha256(b"hello");
        let hash2 = hash_sha256(b"hello");
        assert_eq!(hash1, hash2);
    }

    #[test]
    fn test_different_inputs_produce_different_hashes() {
        let h1 = hash_sha256(b"hello");
        let h2 = hash_sha256(b"world");
        assert_ne!(h1, h2);
    }

    #[test]
    fn test_hash_sha256_known_value() {
        // SHA256("") = e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855
        let hash = hash_sha256(b"");
        assert_eq!(
            hash,
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        );
    }

    #[test]
    fn test_hash_sha256_output_is_64_hex_chars() {
        let hash = hash_sha256(b"test");
        assert_eq!(hash.len(), 64);
        assert!(hash.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn test_check_pow() {
        assert!(check_pow("000abc", 3));
        assert!(!check_pow("0abc", 3));
    }

    #[test]
    fn test_check_pow_difficulty_zero_always_passes() {
        assert!(check_pow("abc123", 0));
        assert!(check_pow("", 0));
    }

    #[test]
    fn test_check_pow_exact_boundary() {
        assert!(check_pow("00", 2));
        assert!(check_pow("000", 2)); // "000" starts with "00", so it passes
        assert!(!check_pow("0", 2));
        assert!(!check_pow("10", 2));
    }

    #[test]
    fn test_hash_object_serializable() {
        let h1 = hash_object(&("alice", "bob", 10.0_f64)).unwrap();
        let h2 = hash_object(&("alice", "bob", 10.0_f64)).unwrap();
        assert_eq!(h1, h2);
    }

    #[test]
    fn test_hash_object_different_values_differ() {
        let h1 = hash_object(&("alice", 10.0_f64)).unwrap();
        let h2 = hash_object(&("alice", 20.0_f64)).unwrap();
        assert_ne!(h1, h2);
    }
}
