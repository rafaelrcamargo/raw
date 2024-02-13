use std::{
    io::prelude::*,
    net::{TcpListener, UdpSocket},
};

use shared::{utils, NewTransaction, SuccessfulTransaction, SIZE};

fn main() {
    let listener = TcpListener::bind("127.0.0.1:7878").unwrap();
    let socket = UdpSocket::bind("127.0.0.1:4243").unwrap();
    dbg!("Server started");

    for stream in listener.incoming() {
        let mut stream = stream.unwrap();

        dbg!("New connection");

        let new = NewTransaction {
            id: 1,
            kind: 'c' as u8,
            value: 100,
            description: utils::to_fixed_slice("yey"),
        };

        dbg!(&new);

        socket
            .send_to(&bincode::serialize(&new).unwrap(), "127.0.0.1:4242")
            .expect("Error on send");

        dbg!("Sent");

        let mut buf = [0; (SIZE as usize) * 10 + 1];
        let amt = socket.recv(&mut buf).unwrap();

        dbg!(amt);

        let resp = bincode::deserialize::<SuccessfulTransaction>(&buf[..amt]).unwrap();
        let json = serde_json::to_string(&resp).unwrap();

        dbg!(&json);

        stream
            .write_all(format!("HTTP/1.1 200 OK\r\nContent-Length: {}\r\n\r\n{}", json.len(), json).as_bytes())
            .unwrap();
    }
}
