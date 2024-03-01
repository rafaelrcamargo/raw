use async_std::{
    io::{ReadExt, WriteExt},
    net::{TcpListener, TcpStream}
};
use futures::stream::StreamExt;
use std::{
    net::Ipv4Addr,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc
    }
};

// Static server addresses
const SERVER_1: (Ipv4Addr, u16) = (Ipv4Addr::new(0, 0, 0, 0), 8080);
const SERVER_2: (Ipv4Addr, u16) = (Ipv4Addr::new(0, 0, 0, 0), 8081);

#[async_std::main]
async fn main() {
    let addr = std::env::var("TCP_PORT").unwrap_or("9999".to_string());
    let listener = TcpListener::bind(format!("0.0.0.0:{}", addr))
        .await
        .unwrap();

    let round_robin = Arc::new(AtomicBool::new(true));

    println!("Server started! (TCP: {})", addr);
    listener
        .incoming()
        .for_each_concurrent(/* limit */ 8, |stream| async {
            let stream = stream.unwrap();
            stream.set_nodelay(true).unwrap();
            forward_stream(stream, round_robin.clone()).await;
        })
        .await;
}

/// Static HTTP 404 response
pub const NOT_FOUND: &[u8] = b"HTTP/1.1 404 Not Found\r\nContent-Length: 0\r\n\r\n";

async fn forward_stream(mut stream: TcpStream, server: Arc<AtomicBool>) {
    let mut buf = [0u8; 1024]; // That's the exact size of a request
    let n = stream.read(&mut buf).await.unwrap();

    match buf[4] {
        32 => {
            let id = buf[15] - b'0'; // This is probably the unsafest safe rust code ever

            if id > 5 {
                stream.write_all(NOT_FOUND).await.unwrap();
                // println!("Invalid ID: {:.2?}", before.elapsed());
                return stream.flush().await.unwrap();
            }
        }
        47 => {
            let id = buf[14] - b'0'; // This is probably the unsafest safe rust code ever

            if id > 5 {
                stream.write_all(NOT_FOUND).await.unwrap();
                // println!("Invalid ID: {:.2?}", before.elapsed());
                return stream.flush().await.unwrap();
            }
        }
        _ => {
            stream.write_all(NOT_FOUND).await.unwrap();
            // println!("Invalid ID: {:.2?}", before.elapsed());
            return stream.flush().await.unwrap();
        }
    }

    let server = server
        .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |x| Some(!x))
        .unwrap();

    match TcpStream::connect(if server { SERVER_1 } else { SERVER_2 }).await {
        Ok(mut inner_stream) => {
            inner_stream.write_all(&buf[0..n]).await.unwrap();
            let mut data = [0u8; 1024];
            match inner_stream.read(&mut data).await {
                Ok(n) => {
                    stream.write_all(&data[0..n]).await.unwrap();
                    stream.flush().await.unwrap();
                }
                Err(e) => panic!("Failed to receive data: {}", e)
            };
        }
        Err(e) => panic!("Failed to connect: {}", e)
    }
}
