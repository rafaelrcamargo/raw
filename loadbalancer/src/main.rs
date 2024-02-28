use async_std::{
    io::{ReadExt, WriteExt},
    net::{TcpListener, TcpStream},
    sync::Mutex
};
use futures::stream::StreamExt;
use std::sync::Arc;

struct RoundRobin<T> {
    items: Arc<Mutex<Vec<T>>>,
    current_index: usize
}

impl<T> RoundRobin<T>
where
    T: Clone
{
    fn new(items: Vec<T>) -> Self {
        RoundRobin {
            items: Arc::new(Mutex::new(items)),
            current_index: 0
        }
    }

    async fn next(&mut self) -> Option<T> {
        let items = self.items.lock().await;
        if items.is_empty() {
            None
        } else {
            let next_item = items[self.current_index].clone();
            self.current_index = (self.current_index + 1) % items.len();
            Some(next_item)
        }
    }
}

#[async_std::main]
async fn main() {
    let addr = std::env::var("TCP_PORT").unwrap_or("9999".to_string());
    let listener = TcpListener::bind(format!("0.0.0.0:{}", addr))
        .await
        .unwrap();

    let items = vec!["8080", "8081"];
    let round_robin = Arc::new(Mutex::new(RoundRobin::new(items)));

    println!("Server started! (TCP: {})", addr);
    listener
        .incoming()
        .for_each_concurrent(/* limit */ 4, |stream| async {
            let stream = stream.unwrap();
            stream.set_nodelay(true).unwrap();
            forward_stream(stream, round_robin.clone()).await;
        })
        .await;
}

async fn forward_stream(mut stream: TcpStream, server: Arc<Mutex<RoundRobin<&str>>>) {
    let mut buf = [0u8; 1024]; // That's the exact size of a request
    let n = stream.read(&mut buf).await.unwrap();

    let addr = String::from("0.0.0.0:") + server.lock().await.next().await.unwrap();

    match TcpStream::connect(addr).await {
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
