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
    pub id: u8,
    pub kind: u8,
    pub value: i32,
    pub description: [u8; 10],
}

impl NewTransaction {
    pub fn to_transaction(&self, limit: u32, balance: i32) -> Transaction {
        Transaction {
            limit,
            balance,
            value: self.value,
            operation: self.kind,
            description: self.description,
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
