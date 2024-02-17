#![allow(unused_must_use)]

use shared::{ClientState, NewTransaction, SuccessfulTransaction, Transaction, TRANSACTION_SIZE};
use smallvec::{smallvec, SmallVec};
use std::{
    fs::{self, File, OpenOptions},
    io::{Read, Seek, SeekFrom, Write},
    net::UdpSocket
};

const DIR: &str = "data/";
fn prepare(cache: &mut SmallVec<[ClientState; 5]>) -> SmallVec<[File; 5]> {
    if fs::metadata(DIR).is_ok() {
        fs::remove_dir_all(DIR).unwrap();
        fs::create_dir(DIR).unwrap();
    }

    [
        (100000, 0, 0, b'c', "init"),
        (80000, 0, 0, b'c', "init"),
        (1000000, 0, 0, b'c', "init"),
        (10000000, 0, 0, b'c', "init"),
        (500000, 0, 0, b'c', "init")
    ]
    .iter()
    .enumerate()
    .map(|(i, x)| {
        let transaction = Transaction::from(*x);
        let id = i + 1;

        cache[i] = ClientState {
            limit: transaction.limit,
            balance: transaction.balance
        };

        fs::create_dir(DIR).unwrap_or_default();

        let mut file = OpenOptions::new()
            .append(true)
            .create(true)
            .read(true)
            .open(format!("{}{}", DIR, id))
            .unwrap();

        let encoded = bincode::serialize(&transaction).unwrap();
        file.write_all(&encoded).unwrap();

        file
    })
    .collect()
}

fn main() {
    let mut cache: SmallVec<[ClientState; 5]> = smallvec![ClientState {
        balance: 0,
        limit: 0,
    }; 5];

    let mut entities = prepare(&mut cache); // Set the initial state

    let socket = UdpSocket::bind("0.0.0.0:4242").unwrap();
    let mut buf = [0; 256]; // Buffer to hold the data

    println!("Database started! (UDP: 4242)");
    loop {
        let (amt, src) = socket.recv_from(&mut buf).unwrap();
        // let before = Instant::now();

        // Data
        let id = buf[0] as usize;

        // Statement
        if amt == 1 {
            socket.send_to(&get(&mut entities[id - 1]), src);
            // println!("GET: {:.2?}", before.elapsed());
            continue;
        };

        // Transaction
        let entity = &mut cache[id - 1];
        let new = bincode::deserialize::<NewTransaction>(&buf[1..amt]).unwrap();

        let transaction = {
            if buf[1] == b'c' {
                new.to_transaction(entity.limit, entity.balance + new.value)
            } else if entity.balance - new.value >= -(entity.limit as i32) {
                new.to_transaction(entity.limit, entity.balance - new.value)
            } else {
                socket.send_to(&[], src);
                // println!("Not allowed: {:.2?}", before.elapsed());
                continue;
            }
        };

        let success = SuccessfulTransaction::from_transaction(&transaction);
        cache[id - 1] = unsafe { std::mem::transmute::<&SuccessfulTransaction, &ClientState>(&success) }.to_owned();

        // println!("Post: {:.2?}", before.elapsed());
        socket.send_to(&bincode::serialize(&success).unwrap(), src);
        insert(&mut entities[id - 1], bincode::serialize(&transaction).unwrap());
        // println!("All: {:.2?}", before.elapsed());
    }
}

fn insert(file: &mut File, data: Vec<u8>) { file.write_all(&data); }

fn get(file: &mut File) -> Vec<u8> {
    let amount = file
        .metadata()
        .unwrap()
        .len()
        .min((TRANSACTION_SIZE as u64) * 10);
    let mut buf = vec![0u8; amount as usize];
    file.seek(SeekFrom::End(-(amount as i64)));
    file.read_exact(&mut buf);
    file.rewind();
    buf
}
