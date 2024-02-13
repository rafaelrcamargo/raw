use std::{
    fs::{self, File, OpenOptions},
    io::{Read, Seek, SeekFrom, Write},
    net::UdpSocket,
};

use serde::{Deserialize, Serialize};

/**
 * I structure for the Transaction, this can be a array, for convenience the "descricao" can be a 10 char string
 * We can keep a in memory shared array of the actual state of the clients, with the id, last balance, and limit
 * we only update the value of the in memory array, and append to a file that we have with the full transaction and timestamp
 * This will work as we only have a fixed amount of clients
 */

fn main() {
    prepare(); // Set the initial state

    let socket = UdpSocket::bind("127.0.0.1:4242").unwrap();
    let mut buf = [0; 32];

    loop {
        let (amt, src) = socket.recv_from(&mut buf).unwrap();

        dbg!(amt, &buf[..amt], &src);

        if amt == 1 {
            socket.send_to("Extrato".as_bytes(), &src).unwrap();
        }

        socket.send_to("Transacao".as_bytes(), &src).unwrap();
    }

    /* buf.chunks(SIZE as usize).for_each(|x| {
        let _: Transaction = bincode::deserialize(x).unwrap();
    }); */
}

#[derive(Serialize, Deserialize, Debug)]
struct Transaction {
    limit: u32,
    balance: i32,
    value: i32,
    operation: u8,
    description: [u8; 10],
    timestamp: u64,
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

const SIZE: u8 = 31;
const DIR: &str = "data/";

fn prepare() {
    vec![
        (100000, 0, 0, 0, ""),
        (80000, 0, 0, 0, ""),
        (1000000, 0, 0, 0, ""),
        (10000000, 0, 0, 0, ""),
        (500000, 0, 0, 0, ""),
    ]
    .iter()
    .enumerate()
    .for_each(|(i, x)| {
        fs::create_dir(DIR).unwrap_or_default();

        let mut file = OpenOptions::new()
            .write(true)
            .truncate(true)
            .create(true)
            .open(format!("{}{}", DIR, i))
            .unwrap();

        let encoded = bincode::serialize(&Transaction::from(*x)).unwrap();
        file.write_all(&encoded).unwrap();
    });
}

mod utils {
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
}

fn insert(id: u8, data: Vec<u8>) {
    OpenOptions::new()
        .append(true)
        .open(format!("{}{}", DIR, id))
        .unwrap()
        .write_all(&data)
        .unwrap()
}

fn get(id: u8) -> Vec<u8> {
    let mut file = File::open(format!("{}{}", DIR, id)).unwrap();
    let amount = file.metadata().unwrap().len().min((SIZE as u64) * 10);
    let mut buf = vec![0u8; amount as usize];
    file.seek(SeekFrom::End(-(amount as i64))).unwrap();
    file.read(&mut buf).unwrap();
    buf
}
