use base64::engine::general_purpose::STANDARD as BASE64;
use base64::Engine;
use hmac::{Hmac, Mac};
use md5::{Digest, Md5};
use rand::Rng;
use serde_json::json;
use sha2::Sha256;
use std::time::{SystemTime, UNIX_EPOCH};
use url::Url;

pub const GATEWAY_SECRET_ONLINE: &str = "76iRl07s0xSN9jqmEWAt79EBJZulIQIsV64FZr2O";
pub const DEFAULT_GUEST_TOKEN: &str = "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJ1aWQiOjcwNjU5NDg0MTAyMTM4MTYyMzIsInV0cCI6MSwiZXhwIjoxNzkxNzMyMjMzLCJpYXQiOjE3ODM5NTU5MzN9.7iyEzTj4vWAbOF0oXwNnZ0p3Nc1QaO6K9eMiGFyVfGs";

pub fn current_timestamp_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}

pub fn md5_hex(data: &[u8]) -> String {
    if data.is_empty() {
        return String::new();
    }
    let mut hasher = Md5::new();
    hasher.update(data);
    format!("{:x}", hasher.finalize())
}

#[allow(dead_code)]
pub fn sha256_hex(data: &[u8]) -> String {
    if data.is_empty() {
        return String::new();
    }
    let mut hasher = Sha256::new();
    hasher.update(data);
    format!("{:x}", hasher.finalize())
}

pub fn generate_client_token() -> String {
    let ts = current_timestamp_ms().to_string();
    let reversed_ts: String = ts.chars().rev().collect();
    let hash = md5_hex(reversed_ts.as_bytes());
    format!("{},{}", ts, hash)
}

pub fn generate_tr_signature(method: &str, raw_url: &str, body_data: &str, timestamp: u64) -> String {
    let parsed = Url::parse(raw_url).unwrap_or_else(|_| Url::parse("http://localhost").unwrap());
    let path = parsed.path();

    let mut query_pairs: Vec<(String, String)> = parsed
        .query_pairs()
        .map(|(k, v)| (k.into_owned(), v.into_owned()))
        .collect();

    let resource = if !query_pairs.is_empty() {
        query_pairs.sort_by(|a, b| a.0.cmp(&b.0));
        let query_string = query_pairs
            .into_iter()
            .map(|(k, v)| format!("{}={}", k, v))
            .collect::<Vec<_>>()
            .join("&");
        format!("{}?{}", path, query_string)
    } else {
        path.to_string()
    };

    let body_md5 = if !body_data.is_empty() {
        let truncated = if body_data.len() > 0x19000 {
            &body_data[..0x19000]
        } else {
            body_data
        };
        md5_hex(truncated.as_bytes())
    } else {
        String::new()
    };

    let accept = "application/json";
    let content_type = "application/json;charset=UTF-8";
    let content_length = if !body_data.is_empty() {
        body_data.len().to_string()
    } else {
        String::new()
    };

    let canonical_string = format!(
        "{}\n{}\n{}\n{}\n{}\n{}\n{}",
        method.to_uppercase(),
        accept,
        content_type,
        content_length,
        timestamp,
        body_md5,
        resource
    );

    let key_bytes = BASE64.decode(GATEWAY_SECRET_ONLINE).unwrap_or_default();
    type HmacMd5 = Hmac<Md5>;
    let mut mac = HmacMd5::new_from_slice(&key_bytes).expect("HMAC can take key of any size");
    mac.update(canonical_string.as_bytes());
    let result = mac.finalize().into_bytes();
    let signature_b64 = BASE64.encode(result);

    format!("{}|2|{}", timestamp, signature_b64)
}

pub fn get_default_client_info() -> serde_json::Value {
    let mut rng = rand::thread_rng();
    let random_digits: String = (0..7).map(|_| rng.gen_range(0..=9).to_string()).collect();
    let device_id = format!("86820305{}", random_digits);

    json!({
        "package_name": "com.community.oneroom",
        "version_name": "4.0.02",
        "version_code": 50020126,
        "os": "android",
        "os_version": "14",
        "install_ch": "ps",
        "device_id": device_id,
        "install_store": "ps",
        "gaid": "",
        "brand": "Google",
        "model": "Pixel 6",
        "system_language": "en",
        "net": "wifi",
        "region": "IN",
        "timezone": "Asia/Kolkata",
        "sp_code": "404"
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_md5() {
        assert_eq!(md5_hex(b"hello"), "5d41402abc4b2a76b9719d911017c592");
    }

    #[test]
    fn test_signature_format() {
        let sig = generate_tr_signature("GET", "https://apig.inmoviebox.com/subject-api/get?subjectId=123", "", 1700000000000);
        assert!(sig.starts_with("1700000000000|2|"));
    }
}
