pub mod utils;

use serde::{Deserialize, Serialize};

pub const SIZE: u8 = 31;

#[derive(Serialize, Deserialize)]
pub struct Transaction {
    pub limit: u32,
    pub balance: i32,
    pub value: i32,
    pub operation: u8,
    pub description: [u8; 10],
    pub timestamp: u64,
}

impl From<(u32, i32, i32, u8, &str)> for Transaction {
    fn from((limit, balance, value, operation, description): (u32, i32, i32, u8, &str)) -> Self {
        Self {
            limit,
            balance,
            value,
            operation,
            description: utils::to_fixed_slice(description),
            timestamp: utils::get_time(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct IncomingTransaction {
    #[serde(rename = "tipo")]
    #[serde(deserialize_with = "deserialize_char_from_string")]
    pub kind: u8,
    #[serde(rename = "valor")]
    pub value: i32,
    #[serde(rename = "descricao")]
    #[serde(with = "serde_bytes")]
    pub description: Vec<u8>,
}

#[derive(Deserialize, Debug)]
pub struct NewTransaction {
    pub kind: u8,
    pub value: i32,
    pub description: Vec<u8>,
}

fn deserialize_char_from_string<'de, D>(deserializer: D) -> Result<u8, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let s: String = Deserialize::deserialize(deserializer)?;

    // Assume the string contains only one character
    if let Some(c) = s.chars().next() {
        Ok(c as u8)
    } else {
        Err(serde::de::Error::custom(
            "Expected a string with exactly one character",
        ))
    }
}

impl NewTransaction {
    pub fn to_transaction(&self, limit: u32, balance: i32) -> Transaction {
        Transaction {
            limit,
            balance,
            value: self.value,
            operation: self.kind,
            description: utils::from_vec_to_fixed_slice(&self.description),
            timestamp: utils::get_time(),
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct SuccessfulTransaction {
    #[serde(rename = "limite")]
    pub limit: u32,
    #[serde(rename = "saldo")]
    pub balance: i32,
}

#[derive(Debug, Copy)]
pub struct ClientState {
    pub limit: u32,
    pub balance: i32,
}

impl Clone for ClientState {
    fn clone(&self) -> Self {
        Self {
            limit: self.limit,
            balance: self.balance,
        }
    }
}
