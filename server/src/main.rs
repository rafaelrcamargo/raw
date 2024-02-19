use std::sync::Arc;

use async_std::{
    io::{ReadExt, WriteExt},
    net::{TcpListener, TcpStream, UdpSocket},
    sync::Mutex
    // task::spawn
};
use futures::stream::StreamExt;
use server::*;
use shared::{IncomingTransaction, SuccessfulTransaction, Transaction, TRANSACTION_SIZE};
use smallvec::SmallVec;

#[async_std::main]
async fn main() {
    // let db_url = format!("{}:4242", std::env::var("DB_URL").unwrap_or("0.0.0.0".to_string()));
    // let socket = UdpSocket::bind("0.0.0.0:4040").unwrap();

    let socket = Arc::new(Mutex::new(UdpSocket::bind("0.0.0.0:4040").await.unwrap()));
    let tcp_port = std::env::var("TCP_PORT").unwrap_or("9999".to_string());
    let listener = TcpListener::bind(format!("0.0.0.0:{}", tcp_port))
        .await
        .unwrap();

    println!("Server started! (TCP: {})", tcp_port);

    listener
        .incoming()
        .for_each_concurrent(/* limit */ 2, |stream| async {
            let stream = unsafe { stream.unwrap_unchecked() };
            unsafe { stream.set_nodelay(true).unwrap_unchecked() };
            handle_request(stream, socket.clone()).await;
        })
        .await;
}

async fn handle_request(mut stream: TcpStream, socket: Arc<Mutex<UdpSocket>>) {
    // let before = std::time::Instant::now();

    let mut buf = [0; 256]; // That's the exact size of a request
    let end = stream.read(&mut buf).await.unwrap();
    // println!("\n\nReceived: {:.2?}", before.elapsed());

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

            println!("POST: {} - {:?}", id, body);

            // println!("Parsed JSON: {:.2?}", before.elapsed());
            let socket = socket.lock().await;

            socket
                .send_to(
                    &[&[id], bincode::serialize(&body).unwrap().as_slice()].concat(),
                    "database:4242"
                )
                .await
                .unwrap();
            // println!("DB Req: {:.2?}", before.elapsed());

            let mut buf = [0; 8];
            let n = socket.recv(&mut buf).await.unwrap();
            // println!("DB Resp: {:.2?}", before.elapsed());

            if n == 0 {
                stream.write_all(UNPROCESSABLE_ENTITY).await.unwrap();
                // println!("Unprocessable entity: {:.2?}", before.elapsed());
                return stream.flush().await.unwrap();
            }

            let resp = bincode::deserialize::<SuccessfulTransaction>(&buf[..n]).unwrap();
            let json = simd_json::to_string(&resp).unwrap();

            // println!("Deserialized: {:.2?}", before.elapsed());

            let mut resp = String::with_capacity(32);
            resp.push_str("HTTP/1.1 200 OK\r\nContent-Length:");
            resp.push_str(&json.len().to_string());
            resp.push_str("\r\n\r\n");
            resp.push_str(&json);
            stream.write_all(resp.as_bytes()).await.unwrap();

            // println!("Sent POST: {:.2?}", before.elapsed());
            return stream.flush().await.unwrap(); // We don't want to close the stream
        }
        47 => {
            let id = buf[14] - b'0';

            if id > 5 {
                stream.write_all(NOT_FOUND).await.unwrap();
                // println!("Invalid ID: {:.2?}", before.elapsed());
                return stream.flush().await.unwrap();
            }

            println!("GET: {}", id);

            let socket = socket.lock().await;
            socket.send_to(&[id], "database:4242").await.unwrap();

            let mut buf = [0; (TRANSACTION_SIZE as usize) * 10];
            let n = socket.recv(&mut buf).await.unwrap();

            let transactions = to_json(
                buf[..n]
                    .chunks(TRANSACTION_SIZE as usize)
                    .map(|x| bincode::deserialize::<Transaction>(x).unwrap())
                    .collect::<SmallVec<[Transaction; 10]>>()
            );

            // println!("Ops. GET: {:.2?}", before.elapsed());

            let mut resp = String::with_capacity(768);
            resp.push_str("HTTP/1.1 200 OK\r\nContent-Length:");
            resp.push_str(&transactions.len().to_string());
            resp.push_str("\r\n\r\n");
            resp.push_str(&transactions);

            stream.write_all(resp.as_bytes()).await.unwrap();
            // println!("Sent GET: {:.2?}", before.elapsed());
            return stream.flush().await.unwrap();
        }
        _ => {
            stream.write_all(NOT_FOUND).await.unwrap();
            // println!("Not found: {:.2?}", before.elapsed());
            return stream.flush().await.unwrap();
        }
    }
}
