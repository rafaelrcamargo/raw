use std::time::SystemTime;

/// Get the current time in seconds
/// - It would only panic if the system time was before the UNIX epoch and then this unsafe is the least of our problems
pub fn get_time() -> u64 {
    unsafe {
        SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap_unchecked()
    }
    .as_secs()
}

/// Convert a string slice into a fixed-size ([u8; 10]) slice
pub fn to_fixed_slice(s: &[u8]) -> [u8; 10] {
    s.try_into().unwrap_or_else(|_| {
        let mut tmp = [0u8; 10];
        tmp[..s.len()].copy_from_slice(s);
        tmp
    })
}
