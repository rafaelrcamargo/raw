use std::time::SystemTime;

pub fn get_time() -> u64 {
    SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

pub fn to_fixed_slice(s: &str) -> [u8; 10] {
    let mut tmp = [0u8; 10];
    tmp[..s.len()].copy_from_slice(s.as_bytes());
    tmp
}

pub fn from_vec_to_fixed_slice(s: &[u8]) -> [u8; 10] {
    let mut tmp = [0u8; 10];
    tmp[..s.len()].copy_from_slice(s);
    tmp
}
