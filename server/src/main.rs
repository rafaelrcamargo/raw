use async_std::{
    io::{ReadExt, WriteExt},
    net::{TcpListener, TcpStream, UdpSocket},
    sync::Mutex
};
use bincode::config;
use futures::stream::StreamExt;
use server::*;
use shared::{IncomingTransaction, SuccessfulTransaction, Transaction, TRANSACTION_SIZE};
use smallvec::SmallVec;
use std::{net::Ipv4Addr, sync::Arc};

const DB_ADDR: (Ipv4Addr, u16) = (Ipv4Addr::new(0, 0, 0, 0), 4242);

#[async_std::main]
async fn main() {
    let addr = std::env::var("UDP_PORT").unwrap_or("4040".to_string());
    let socket = Arc::new(Mutex::new(UdpSocket::bind(format!("0.0.0.0:{}", addr)).await.unwrap()));

    let addr = std::env::var("TCP_PORT").unwrap_or("8080".to_string());
    let listener = TcpListener::bind(format!("0.0.0.0:{}", addr))
        .await
        .unwrap();

    println!("Server started! (TCP: {})", addr);
    listener
        .incoming()
        .for_each_concurrent(/* limit */ 2, |stream| async {
            let stream = stream.unwrap();
            stream.set_nodelay(true).unwrap();
            handle_request(stream, socket.clone()).await;
        })
        .await
}

async fn handle_request(mut stream: TcpStream, socket: Arc<Mutex<UdpSocket>>) {
    let mut buf = vec![0; 224].into_boxed_slice(); // That's the exact size of a request
    let end = stream.read(&mut buf).await.unwrap();
    // println!("Received: {} - {:.2?}", end, before.elapsed());

    match buf[4] {
        32 => {
            let id = buf[15] - b'0'; // This is probably the unsafest safe rust code ever

            if id > 5 {
                stream.write_all(NOT_FOUND).await.unwrap();
                // println!("Invalid ID: {:.2?}", before.elapsed());
                return stream.flush().await.unwrap();
            }

            let start = find_subsequence(&buf, b"\r\n\r\n").unwrap() + 4;
            let body = &mut buf[start..end];
            // println!("Parsed: {:.2?}", before.elapsed());

            let body = match simd_json::from_slice::<IncomingTransaction>(body) {
                Ok(body) => body,
                Err(_) => {
                    stream.write_all(UNPROCESSABLE_ENTITY).await.unwrap();
                    // println!("Unprocessable entity: {:.2?}", before.elapsed());
                    return stream.flush().await.unwrap();
                }
            };
            // println!("Parsed JSON: {:.2?}", before.elapsed());

            let socket = socket.lock().await;

            socket
                .send_to(
                    &[
                        &[id],
                        bincode::serde::encode_to_vec(&body, config::legacy())
                            .unwrap()
                            .as_slice()
                    ]
                    .concat(),
                    DB_ADDR
                )
                .await
                .unwrap();
            // println!("DB Req: {:.2?}", before.elapsed());

            let mut buf = [0; 8];
            let n = socket.recv(&mut buf).await.unwrap();
            // println!("DB Resp: {} - {:.2?}", n, before.elapsed());

            if n == 0 {
                stream.write_all(UNPROCESSABLE_ENTITY).await.unwrap();
                // println!("Unprocessable entity: {:.2?}", before.elapsed());
                return stream.flush().await.unwrap();
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
            stream.write_all(resp.as_bytes()).await.unwrap();
            // println!("Sent POST: {:.2?}", before.elapsed());
            stream.flush().await.unwrap()
        }
        47 => {
            let id = buf[14] - b'0';

            if id > 5 {
                stream.write_all(NOT_FOUND).await.unwrap();
                // println!("Invalid ID: {:.2?}", before.elapsed());
                return stream.flush().await.unwrap();
            }

            let socket = socket.lock().await;

            socket.send_to(&[id], DB_ADDR).await.unwrap();

            let mut buf = [0; (TRANSACTION_SIZE as usize) * 10];
            let n = socket.recv(&mut buf).await.unwrap();

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
            stream.write_all(resp.as_bytes()).await.unwrap();
            // println!("Sent GET: {:.2?}", before.elapsed());
            stream.flush().await.unwrap()
        }
        _ => {
            stream.write_all(NOT_FOUND).await.unwrap();
            // println!("Not found: {:.2?}", before.elapsed());
            stream.flush().await.unwrap()
        }
    }
}
