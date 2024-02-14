pub mod utils;

use serde::{Deserialize, Serialize};

pub const SIZE: u8 = 31;

#[derive(Serialize, Deserialize, Debug)]
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
pub struct NewTransaction {
    #[serde(rename = "tipo")]
    #[serde(deserialize_with = "deserialize_char_from_string")]
    pub kind: u8,
    #[serde(rename = "valor")]
    pub value: i32,
    #[serde(rename = "descricao")]
    #[serde(with = "serde_bytes")]
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
            description: utils::to_fixed_slice(
                &String::from_utf8_lossy(&self.description).replace('\0', ""),
            ),
            timestamp: utils::get_time(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SuccessfulTransaction {
    pub limit: u32,
    pub balance: i32,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ClientState {
    pub limit: u32,
    pub balance: i32,
}
