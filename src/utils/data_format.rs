/// 将字节数据转换为十六进制字符串
pub fn bytes_to_hex(bytes: &[u8]) -> String {
    bytes
        .iter()
        .map(|b| format!("{:02X}", b))
        .collect::<Vec<String>>()
        .join(" ")
}

/// 将十六进制字符串转换为字节数据
pub fn hex_to_bytes(hex_str: &str) -> Result<Vec<u8>, String> {
    // 移除所有空格
    let hex_str = hex_str.replace(' ', "");
    
    // 验证字符串长度是偶数
    if hex_str.len() % 2 != 0 {
        return Err("Invalid hex string length".to_string());
    }
    
    // 将每两个字符转换为一个字节
    let mut bytes = Vec::new();
    for i in (0..hex_str.len()).step_by(2) {
        let byte_str = &hex_str[i..i + 2];
        match u8::from_str_radix(byte_str, 16) {
            Ok(byte) => bytes.push(byte),
            Err(_) => return Err(format!("Invalid hex characters: {}", byte_str)),
        }
    }
    
    Ok(bytes)
}

/// 将字符串转换为字节数据
pub fn string_to_bytes(s: &str) -> Vec<u8> {
    s.as_bytes().to_vec()
}

/// 将字节数据转换为字符串
pub fn bytes_to_string(bytes: &[u8]) -> String {
    String::from_utf8_lossy(bytes).to_string()
}

/// 格式化JSON字符串
pub fn format_json(json_str: &str) -> String {
    match serde_json::from_str::<serde_json::Value>(json_str) {
        Ok(json) => serde_json::to_string_pretty(&json).unwrap_or_else(|_| json_str.to_string()),
        Err(_) => json_str.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hex_conversion() {
        let original = vec![0x01, 0x02, 0xAB, 0xFF];
        let hex = bytes_to_hex(&original);
        assert_eq!(hex, "01 02 AB FF");

        let bytes = hex_to_bytes(&hex).unwrap();
        assert_eq!(bytes, original);
    }

    #[test]
    fn test_hex_conversion_no_spaces() {
        let hex = "0102ABFF";
        let bytes = hex_to_bytes(hex).unwrap();
        assert_eq!(bytes, vec![0x01, 0x02, 0xAB, 0xFF]);
    }

    #[test]
    fn test_invalid_hex() {
        assert!(hex_to_bytes("0102ABFG").is_err());
        assert!(hex_to_bytes("0102ABF").is_err());
    }

    #[test]
    fn test_hex_with_spaces() {
        let hex = "01 02 AB FF";
        let bytes = hex_to_bytes(hex).unwrap();
        assert_eq!(bytes, vec![0x01, 0x02, 0xAB, 0xFF]);
    }

    #[test]
    fn test_empty_hex() {
        assert_eq!(hex_to_bytes("").unwrap(), Vec::<u8>::new());
        assert_eq!(hex_to_bytes("  ").unwrap(), Vec::<u8>::new());
    }

    #[test]
    fn test_format_json_valid() {
        let json = r#"{"key":"value"}"#;
        let formatted = format_json(json);
        assert!(formatted.contains('\n'));
    }

    #[test]
    fn test_format_json_invalid() {
        let bad_json = "not json";
        assert_eq!(format_json(bad_json), "not json");
    }

    #[test]
    fn test_string_bytes_roundtrip() {
        let original = "Hello, 世界!";
        let bytes = string_to_bytes(original);
        let back = bytes_to_string(&bytes);
        assert_eq!(back, original);
    }

    #[test]
    fn test_bytes_to_hex_empty() {
        assert_eq!(bytes_to_hex(&[]), "");
    }
}