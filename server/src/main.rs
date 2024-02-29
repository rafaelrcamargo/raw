#[global_allocator]
static ALLOC: snmalloc_rs::SnMalloc = snmalloc_rs::SnMalloc;

use bincode::config;
use server::*;
use shared::{IncomingTransaction, SuccessfulTransaction, Transaction, TRANSACTION_SIZE};
use smallvec::SmallVec;
use std::{
    io::prelude::*,
    net::{TcpListener, UdpSocket}
};

fn main() {
    let addr = std::env::var("UDP_PORT").unwrap_or("4040".to_string());
    let socket = UdpSocket::bind(format!("0.0.0.0:{}", addr)).unwrap();

    let addr = std::env::var("TCP_PORT").unwrap_or("8080".to_string());
    let listener = TcpListener::bind(format!("0.0.0.0:{}", addr)).unwrap();

    let db_addr = format!("{}:4242", std::env::var("DB_URL").unwrap_or("0.0.0.0".to_string()));

    println!("Server started! (TCP: {})", addr);
    for stream in listener.incoming() {
        // let before = Instant::now();

        let mut stream = stream.unwrap();
        let _ = stream.set_nodelay(true);

        let mut buf = vec![0; 224].into_boxed_slice(); // That's the exact size of a request
        let end = stream.read(&mut buf).unwrap();
        // println!("Received: {} - {:.2?}", end, before.elapsed());

        match buf[4] {
            32 => {
                let id = buf[15] - b'0'; // This is probably the unsafest safe rust code ever

                if id > 5 {
                    stream.write_all(NOT_FOUND).unwrap();
                    // println!("Invalid ID: {:.2?}", before.elapsed());
                    continue;
                }

                let start = find_subsequence(&buf, b"\r\n\r\n").unwrap() + 4;
                let body = &mut buf[start..end];
                // println!("Parsed: {:.2?}", before.elapsed());

                let body = match simd_json::from_slice::<IncomingTransaction>(body) {
                    Ok(body) => body,
                    Err(_) => {
                        stream.write_all(UNPROCESSABLE_ENTITY).unwrap();
                        // println!("Unprocessable entity: {:.2?}", before.elapsed());
                        continue;
                    }
                };
                // println!("Parsed JSON: {:.2?}", before.elapsed());

                socket
                    .send_to(
                        &[
                            &[id],
                            bincode::serde::encode_to_vec(&body, config::legacy())
                                .unwrap()
                                .as_slice()
                        ]
                        .concat(),
                        &db_addr
                    )
                    .unwrap();
                // println!("DB Req: {:.2?}", before.elapsed());

                let mut buf = [0; 8];
                let n = socket.recv(&mut buf).unwrap();
                // println!("DB Resp: {} - {:.2?}", n, before.elapsed());

                if n == 0 {
                    stream.write_all(UNPROCESSABLE_ENTITY).unwrap();
                    // println!("Unprocessable entity: {:.2?}", before.elapsed());
                    continue;
                }

                let (resp, _): (SuccessfulTransaction, usize) =
                    bincode::serde::decode_from_slice(&buf[..n], config::legacy()).unwrap();
                let json = simd_json::to_string(&resp).unwrap();

                // println!("Deserialized: {} {:.2?}", json.len(), before.elapsed());

                let mut resp = String::with_capacity(96);
                resp.push_str("HTTP/1.1 200 OK\r\nContent-Length:");
                resp.push_str(&json.len().to_string());
                resp.push_str("\r\n\r\n");
                resp.push_str(&json);
                stream.write_all(resp.as_bytes()).unwrap();
                // println!("Sent POST: {:.2?}", before.elapsed());
                continue; // We don't want to close the stream
            }
            47 => {
                let id = buf[14] - b'0';

                if id > 5 {
                    stream.write_all(NOT_FOUND).unwrap();
                    // println!("Invalid ID: {:.2?}", before.elapsed());
                    continue;
                }

                socket.send_to(&[id], &db_addr).unwrap();

                let mut buf = [0; (TRANSACTION_SIZE as usize) * 10];
                let n = socket.recv(&mut buf).unwrap();

                let transactions = to_json(
                    buf[..n]
                        .chunks(TRANSACTION_SIZE as usize)
                        .map(|x| {
                            bincode::serde::decode_from_slice::<Transaction, _>(x, config::legacy())
                                .unwrap()
                                .0
                        })
                        .collect::<SmallVec<Transaction, 10>>()
                );

                // println!("Ops. GET: {} - {:.2?}", transactions.len(), before.elapsed());

                let mut resp = String::with_capacity(960);
                resp.push_str("HTTP/1.1 200 OK\r\nContent-Length:");
                resp.push_str(&transactions.len().to_string());
                resp.push_str("\r\n\r\n");
                resp.push_str(&transactions);
                stream.write_all(resp.as_bytes()).unwrap();
                // println!("Sent GET: {:.2?}", before.elapsed());
                continue; // We don't want to close the stream
            }
            _ => {
                stream.write_all(NOT_FOUND).unwrap();
                // println!("Not found: {:.2?}", before.elapsed());
                continue; // We don't want to close the stream
            }
        }
    }
}
