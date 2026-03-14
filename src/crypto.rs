use sha2::{Sha256, Digest};

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
    fn test_check_pow() {
        assert!(check_pow("000abc", 3));
        assert!(!check_pow("0abc", 3));
    }
}
