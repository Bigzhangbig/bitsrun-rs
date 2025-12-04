//! 加密算法实现模块
//!
//! 本模块实现了 SRUN 认证系统使用的加密算法：
//! - xencode: 使用 XXTEA 算法对认证数据进行加密
//! - fkbase64: 使用自定义字母表的 Base64 编码
//! - mix/splite: 辅助函数，用于数据的分块和重组
//!
//! 这些加密函数用于构造登录请求中的加密参数，确保认证信息的安全传输。
//!
//! Encryption Algorithm Implementation Module
//!
//! This module implements encryption algorithms used by SRUN authentication system:
//! - xencode: Encrypts authentication data using XXTEA algorithm
//! - fkbase64: Base64 encoding with custom alphabet
//! - mix/splite: Helper functions for data chunking and reassembly
//!
//! These encryption functions are used to construct encrypted parameters in login requests,
//! ensuring secure transmission of authentication information.
//!
/**
 * Encryption algorithm implementation borrowed from
 * https://github.com/zu1k/srun/blob/d47cd60b54503992ffb4eabeb23b27aecb1edf23/src/xencode.rs
 */
use base64::alphabet::Alphabet;
use base64::engine::Engine;
use base64::engine::GeneralPurpose;
use base64::engine::GeneralPurposeConfig;

const BASE64_ALPHABET: &str = "LVoJPiCN2R8G90yg+hmFHuacZ1OWMnrsSTXkYpUq/3dlbfKwv6xztjI7DeBE45QA";

/// 将字节数组按 4 字节分块并转换为 u32 数组
/// 
/// Convert byte array into u32 chunks (4 bytes per chunk)
///
/// # Arguments
/// * `buffer` - 输入字节数组 / Input byte array
/// * `append_size` - 是否在结果末尾追加原始长度 / Whether to append original size at the end
fn mix(buffer: &[u8], append_size: bool) -> Vec<u32> {
    let mut res: Vec<u32> = buffer
        .chunks(4)
        .map(|chunk| {
            u32::from_le_bytes(chunk.try_into().unwrap_or_else(|_| {
                let mut last_chunk = [0u8, 0, 0, 0];
                last_chunk[..chunk.len()].clone_from_slice(chunk);
                last_chunk
            }))
        })
        .collect();
    if append_size {
        res.push(buffer.len() as u32);
    }
    res
}

/// 将 u32 数组转换回字节数组
/// 
/// Convert u32 array back to byte array
///
/// # Arguments
/// * `buffer` - u32 数组 / u32 array
/// * `include_size` - 是否包含尾部的大小信息 / Whether to include size information at the end
fn splite(buffer: Vec<u32>, include_size: bool) -> Vec<u8> {
    let len = buffer.len();
    let size_record = buffer[len - 1];
    if include_size {
        let size = ((len - 1) * 4) as u32;
        if size_record < size - 3 || size_record > size {
            return "".into();
        }
    }

    let mut buffer: Vec<u8> = buffer.iter().flat_map(|i| i.to_le_bytes()).collect();
    if include_size {
        buffer.truncate(size_record as usize);
    }
    buffer
}

/// 使用 XXTEA 算法加密消息
/// 
/// Encrypt message using XXTEA algorithm
///
/// # Arguments
/// * `msg` - 要加密的消息字符串 / Message string to encrypt
/// * `key` - 加密密钥字符串 / Encryption key string
///
/// # Returns
/// 加密后的字节数组 / Encrypted byte array
pub fn xencode(msg: &str, key: &str) -> Vec<u8> {
    if msg.is_empty() {
        return vec![];
    }
    let mut msg = mix(msg.as_bytes(), true);
    let key = mix(key.as_bytes(), false);

    let len = msg.len();
    let last = len - 1;
    let mut right = msg[last];
    let c: u32 = 0x9e3779b9; // 0x9e3779b9 = 0x86014019 | 0x183639A0
    let mut d: u32 = 0;

    let count = 6 + 52 / msg.len();
    for _ in 0..count {
        d = d.wrapping_add(c);
        let e = d >> 2 & 3;
        for p in 0..=last {
            let left = msg[(p + 1) % len];
            right = ((right >> 5) ^ (left << 2))
                .wrapping_add((left >> 3 ^ right << 4) ^ (d ^ left))
                .wrapping_add(key[(p & 3) ^ e as usize] ^ right)
                .wrapping_add(msg[p]);
            msg[p] = right;
        }
    }
    splite(msg, false)
}

/// 使用自定义字母表进行 Base64 编码
/// 
/// Perform Base64 encoding with custom alphabet
///
/// # Arguments
/// * `payload` - 要编码的字节数组 / Byte array to encode
///
/// # Returns
/// Base64 编码后的字符串 / Base64 encoded string
pub fn fkbase64(payload: Vec<u8>) -> String {
    let alphabet = Alphabet::new(BASE64_ALPHABET).unwrap();
    let engine = GeneralPurpose::new(&alphabet, GeneralPurposeConfig::new());

    engine.encode(payload)
}
