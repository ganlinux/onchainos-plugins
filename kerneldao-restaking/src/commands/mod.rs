pub mod balance;
pub mod stake;
pub mod stake_native;
pub mod unstake;
pub mod unstake_native;

/// Encode ABI calldata for functions with signature: fn(address, uint256, string)
/// Used by stake() and unstake().
/// string is at offset 0x60 (3rd param, after two 32-byte head slots for address+uint256).
pub fn encode_addr_uint_string(selector: &str, addr: &str, amount: u128, s: &str) -> String {
    let addr_stripped = addr.strip_prefix("0x").unwrap_or(addr);
    let addr_padded = format!("{:0>64}", addr_stripped.to_lowercase());
    let amount_hex = format!("{:064x}", amount);
    // String offset: 3 * 32 = 96 = 0x60
    let str_offset = format!("{:064x}", 0x60u64);
    // String ABI encoding: length + data (padded to 32 bytes)
    let str_bytes = s.as_bytes();
    let str_len = format!("{:064x}", str_bytes.len());
    let str_data = if str_bytes.is_empty() {
        String::new()
    } else {
        // pad to 32-byte boundary
        let padded_len = (str_bytes.len() + 31) / 32 * 32;
        let mut padded = vec![0u8; padded_len];
        padded[..str_bytes.len()].copy_from_slice(str_bytes);
        hex::encode(padded)
    };

    format!(
        "0x{selector}{addr_padded}{amount_hex}{str_offset}{str_len}{str_data}",
        selector = selector.strip_prefix("0x").unwrap_or(selector),
    )
}

/// Encode ABI calldata for functions with signature: fn(uint256, string)
/// Used by unstakeNative().
/// String is at offset 0x40 (2nd param, after one 32-byte head slot for uint256).
pub fn encode_uint_string(selector: &str, amount: u128, s: &str) -> String {
    let amount_hex = format!("{:064x}", amount);
    // String offset: 2 * 32 = 64 = 0x40
    let str_offset = format!("{:064x}", 0x40u64);
    let str_bytes = s.as_bytes();
    let str_len = format!("{:064x}", str_bytes.len());
    let str_data = if str_bytes.is_empty() {
        String::new()
    } else {
        let padded_len = (str_bytes.len() + 31) / 32 * 32;
        let mut padded = vec![0u8; padded_len];
        padded[..str_bytes.len()].copy_from_slice(str_bytes);
        hex::encode(padded)
    };

    format!(
        "0x{selector}{amount_hex}{str_offset}{str_len}{str_data}",
        selector = selector.strip_prefix("0x").unwrap_or(selector),
    )
}

/// Encode ABI calldata for functions with signature: fn(string)
/// Used by stakeNative().
/// String is at offset 0x20 (1st param, offset points past the 32-byte head).
pub fn encode_string(selector: &str, s: &str) -> String {
    // String offset: 1 * 32 = 32 = 0x20
    let str_offset = format!("{:064x}", 0x20u64);
    let str_bytes = s.as_bytes();
    let str_len = format!("{:064x}", str_bytes.len());
    let str_data = if str_bytes.is_empty() {
        String::new()
    } else {
        let padded_len = (str_bytes.len() + 31) / 32 * 32;
        let mut padded = vec![0u8; padded_len];
        padded[..str_bytes.len()].copy_from_slice(str_bytes);
        hex::encode(padded)
    };

    format!(
        "0x{selector}{str_offset}{str_len}{str_data}",
        selector = selector.strip_prefix("0x").unwrap_or(selector),
    )
}
