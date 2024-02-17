#![allow(unused_must_use)]

use server::*;
use shared::{IncomingTransaction, SuccessfulTransaction, Transaction, TRANSACTION_SIZE};
use smallvec::SmallVec;
use std::{
    io::prelude::*,
    net::{TcpListener, UdpSocket}
};

fn main() {
    let db_url = format!("{}:4242", std::env::var("DB_URL").unwrap_or("0.0.0.0".to_string()));

    let tcp_port = std::env::var("TCP_PORT").unwrap_or("9999".to_string());
    let listener = TcpListener::bind(format!("0.0.0.0:{}", tcp_port)).unwrap();
    let socket = UdpSocket::bind("0.0.0.0:4040").unwrap();

    println!("Server started! (TCP: {})", tcp_port);

    for stream in listener.incoming() {
        // let before = Instant::now();

        let mut stream = unsafe { stream.unwrap_unchecked() };
        unsafe { stream.set_nodelay(true).unwrap_unchecked() };

        let mut buf = [0; 224]; // That's the exact size of a request
        let end = stream.read(&mut buf).expect("This has a 1000 ways to fail");
        // println!("Received: {} - {:.2?}", end, before.elapsed());

        match buf[4] {
            32 => {
                let id = buf[15] - b'0'; // This is probably the unsafest safe rust code ever

                if id > 5 {
                    stream.write_all(NOT_FOUND);
                    // println!("Invalid ID: {:.2?}", before.elapsed());
                    continue;
                }

                let start = find_subsequence(&buf, b"\r\n\r\n").unwrap() + 4;
                let body = &mut buf[start..end];
                // println!("Parsed: {:.2?}", before.elapsed());

                let body = match simd_json::from_slice::<IncomingTransaction>(body) {
                    Ok(body) => body,
                    Err(_) => {
                        stream.write_all(UNPROCESSABLE_ENTITY);
                        // println!("Unprocessable entity: {:.2?}", before.elapsed());
                        continue;
                    }
                };
                // println!("Parsed JSON: {:.2?}", before.elapsed());

                socket.send_to(
                    &[&[id], bincode::serialize(&body).unwrap().as_slice()].concat(),
                    &db_url
                );
                // println!("DB Req: {:.2?}", before.elapsed());

                let mut buf = [0; 8];
                let amt = socket.recv(&mut buf).unwrap();
                // println!("DB Resp: {} - {:.2?}", amt, before.elapsed());

                if amt == 0 {
                    stream.write_all(UNPROCESSABLE_ENTITY);
                    // println!("Unprocessable entity: {:?}", buf);
                    // println!("Unprocessable entity: {:.2?}", before.elapsed());
                    continue;
                }

                let resp = bincode::deserialize::<SuccessfulTransaction>(&buf[..amt]).unwrap();
                let json = simd_json::to_string(&resp).unwrap();

                // println!("Deserialized: {} {:.2?}", json.len(), before.elapsed());

                let mut resp = String::with_capacity(32);
                resp.push_str("HTTP/1.1 200 OK\r\nContent-Length:");
                resp.push_str(&json.len().to_string());
                resp.push_str("\r\n\r\n");
                resp.push_str(&json);
                stream.write_all(resp.as_bytes());

                // println!("Sent POST: {:.2?}", before.elapsed());
                continue; // We don't want to close the stream
            }
            47 => {
                let id = buf[14] - b'0';

                if id > 5 {
                    stream.write_all(NOT_FOUND);
                    // println!("Invalid ID: {:.2?}", before.elapsed());
                    continue;
                }

                socket
                    .send_to(&[id], &db_url)
                    .expect("Error sending to database");

                let mut buf = [0; (TRANSACTION_SIZE as usize) * 10];
                let amt = socket.recv(&mut buf).unwrap();

                let transactions = to_json(
                    buf[..amt]
                        .chunks(TRANSACTION_SIZE as usize)
                        .map(|x| bincode::deserialize::<Transaction>(x).unwrap())
                        .collect::<SmallVec<[Transaction; 10]>>()
                );

                // println!("Ops. GET: {} - {:.2?}",transactions.len(),before.elapsed());

                let mut resp = String::with_capacity(768);
                resp.push_str("HTTP/1.1 200 OK\r\nContent-Length:");
                resp.push_str(&transactions.len().to_string());
                resp.push_str("\r\n\r\n");
                resp.push_str(&transactions);

                stream.write_all(resp.as_bytes());
                // println!("Sent GET: {:.2?}", before.elapsed());
                continue; // We don't want to close the stream
            }
            _ => {
                stream.write_all(NOT_FOUND);
                // println!("Not found: {:.2?}", before.elapsed());
                continue; // We don't want to close the stream
            }
        }
    }
}
