use serde::{Deserialize, Serialize};

pub mod utils;

/// Encoded transaction size
pub const TRANSACTION_SIZE: u8 = 31;
#[derive(Serialize, Deserialize)]
pub struct Transaction {
    pub limit: u32,
    pub balance: i32,
    pub value: i32,
    pub operation: u8,
    pub description: [u8; 10],
    pub timestamp: u64
}

impl From<(u32, i32, i32, u8, &str)> for Transaction {
    fn from((limit, balance, value, operation, description): (u32, i32, i32, u8, &str)) -> Self {
        Self {
            limit,
            balance,
            value,
            operation,
            description: utils::to_fixed_slice(description.as_bytes()),
            timestamp: utils::get_time()
        }
    }
}

/**
 * Here we create two "seemingly" identical structs, but:
 * - IncomingTransaction is used to deserialize the incoming data from the client and map the JSON fields
 * - NewTransaction receives the data in the expected format so there's no need for the serde mappings
 */

#[derive(Serialize, Deserialize, Debug)]
pub struct IncomingTransaction {
    #[serde(rename = "tipo")]
    #[serde(deserialize_with = "deserialize_char_from_string")]
    pub kind: u8,
    #[serde(rename = "valor")]
    pub value: i32,
    #[serde(rename = "descricao")]
    #[serde(deserialize_with = "deserialize_slice_from_string")]
    pub description: [u8; 10]
}

#[derive(Deserialize, Debug)]
pub struct NewTransaction {
    pub kind: u8,
    pub value: i32,
    pub description: [u8; 10]
}

/// This grabs the first byte from a string
fn deserialize_char_from_string<'de, D>(deserializer: D) -> Result<u8, D::Error>
where
    D: serde::Deserializer<'de>
{
    let s: String = Deserialize::deserialize(deserializer).map_err(serde::de::Error::custom)?;
    // Assume the string contains only one character
    match s.bytes().next() {
        Some(c) => {
            if c != b'c' && c != b'd' {
                return Err(serde::de::Error::custom("Expected a string with only 'c' or 'd'"));
            }
            Ok(c)
        }
        None => Err(serde::de::Error::custom(
            "Expected a string with at least one character"
        ))
    }
}
/// This grabs or pads a string to a fixed [u8; 10] slice
fn deserialize_slice_from_string<'de, D>(deserializer: D) -> Result<[u8; 10], D::Error>
where
    D: serde::Deserializer<'de>
{
    let s: String = Deserialize::deserialize(deserializer).map_err(serde::de::Error::custom)?;
    if s.is_empty() || s.len() > 10 {
        return Err(serde::de::Error::custom("Expected a string with at most 10 characters"));
    }
    // Assume the string contains 10 characters or less
    Ok(utils::to_fixed_slice(s.as_bytes()))
}

impl NewTransaction {
    pub fn to_transaction(&self, limit: u32, balance: i32) -> Transaction {
        Transaction {
            limit,
            balance,
            value: self.value,
            operation: self.kind,
            description: self.description,
            timestamp: utils::get_time()
        }
    }
}

/**
 * This two too are seemingly identical, but:
 * - SuccessfulTransaction is used to serialize the data and rename the fields to match the expected JSON
 * - ClientState is used to store the data in the cache and to be able to clone it
 */

#[derive(Serialize, Deserialize)]
pub struct SuccessfulTransaction {
    #[serde(rename = "limite")]
    pub limit: u32,
    #[serde(rename = "saldo")]
    pub balance: i32
}

impl SuccessfulTransaction {
    pub fn from_transaction(transaction: &Transaction) -> Self {
        Self {
            limit: transaction.limit,
            balance: transaction.balance
        }
    }
}

#[derive(Debug, Copy)]
pub struct ClientState {
    pub limit: u32,
    pub balance: i32
}

impl From<(u32, i32)> for ClientState {
    fn from((limit, balance): (u32, i32)) -> Self { Self { limit, balance } }
}

impl Clone for ClientState {
    fn clone(&self) -> Self { *self }
}
