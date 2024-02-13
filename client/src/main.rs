use shared::{utils, NewTransaction, Transaction, SIZE};
use std::{
    io::{self, BufRead},
    net::UdpSocket,
};

fn main() -> std::io::Result<()> {
    let socket = UdpSocket::bind("127.0.0.1:4243")?;

    let stdin = io::stdin();
    for line in stdin.lock().lines() {
        let new = NewTransaction {
            id: 1,
            kind: 'c' as u8,
            value: 100,
            description: utils::to_fixed_slice(&line.unwrap()),
        };

        socket
            .send_to(&bincode::serialize(&new).unwrap(), "127.0.0.1:4242")
            .expect("Error on send");

        let mut buf = [0; (SIZE as usize) * 10 + 1];
        let amt = socket.recv(&mut buf)?;

        buf[..amt].chunks(SIZE as usize).for_each(|x| {
            dbg!(bincode::deserialize::<Transaction>(x).unwrap());
        });
    }
    Ok(())
}
