use std::{
    collections::HashMap,
    fs::{self, File, OpenOptions},
    io::{Read, Seek, SeekFrom, Write},
    net::UdpSocket,
};

use shared::{ClientState, NewTransaction, SuccessfulTransaction, Transaction, SIZE};

const DIR: &str = "data/";
fn prepare(cache: &mut HashMap<u8, ClientState>) -> Vec<File> {
    if fs::metadata(DIR).is_ok() {
        fs::remove_dir_all(DIR).unwrap();
        fs::create_dir(DIR).unwrap();
    }

    vec![
        (100000, 0, 0, 0, ""),
        (80000, 0, 0, 0, ""),
        (1000000, 0, 0, 0, ""),
        (10000000, 0, 0, 0, ""),
        (500000, 0, 0, 0, ""),
    ]
    .iter()
    .enumerate()
    .map(|(i, x)| {
        let transaction = Transaction::from(*x);

        cache.insert(
            i as u8,
            ClientState {
                limit: transaction.limit,
                balance: transaction.balance,
            },
        );

        fs::create_dir(DIR).unwrap_or_default();

        let mut file = OpenOptions::new()
            .read(true)
            .append(true)
            .create(true)
            .open(format!("{}{}", DIR, i + 1))
            .unwrap();

        let encoded = bincode::serialize(&transaction).unwrap();
        file.write_all(&encoded).unwrap();

        file
    })
    .collect::<Vec<File>>()
}

fn main() {
    let mut cache = HashMap::<u8, ClientState>::new();
    let mut entities = prepare(&mut cache); // Set the initial state

    /* UDP Socket */
    let socket = UdpSocket::bind("127.0.0.1:4242").unwrap();
    let mut buf = [0; 32]; // Buffer to hold the data

    loop {
        let (amt, src) = socket.recv_from(&mut buf).unwrap();

        // Data
        let id = buf[0] as usize;

        // Statement
        if amt == 1 {
            socket.send_to(&get(&mut entities[id]), &src).unwrap();
        };

        // Transaction
        let entity = &mut cache.get_mut(&(id as u8)).unwrap();
        let transaction = bincode::deserialize::<NewTransaction>(&buf).unwrap();
        let transaction = {
            dbg!(&entity, &transaction);

            if (buf[1] as char) == 'c' {
                Some(transaction.to_transaction(entity.limit, entity.balance + transaction.value))
            } else if entity.balance - transaction.value < -(entity.limit as i32) {
                Some(transaction.to_transaction(entity.limit, entity.balance - transaction.value))
            } else {
                None
            }
        };

        if transaction.is_none() {
            socket.send_to("Limite excedido".as_bytes(), &src).unwrap();
            continue;
        }

        let transaction = transaction.unwrap();
        let success = SuccessfulTransaction {
            limit: transaction.limit,
            balance: transaction.balance,
        };

        cache.insert(
            id as u8,
            ClientState {
                limit: transaction.limit,
                balance: transaction.balance,
            },
        );

        socket
            .send_to(&bincode::serialize(&success).unwrap(), &src)
            .unwrap();

        insert(&mut entities[id], bincode::serialize(&transaction).unwrap());
    }
}

fn insert(file: &mut File, data: Vec<u8>) {
    file.write_all(&data).unwrap()
}

fn get(file: &mut File) -> Vec<u8> {
    let amount = file.metadata().unwrap().len().min((SIZE as u64) * 10);
    let mut buf = vec![0u8; amount as usize];
    file.seek(SeekFrom::End(-(amount as i64))).unwrap();
    file.read(&mut buf).unwrap();
    buf
}
