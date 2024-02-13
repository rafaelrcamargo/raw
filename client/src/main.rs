use std::io::{self, BufRead};
use std::net::UdpSocket;
use std::str;

fn main() -> std::io::Result<()> {
    let socket = UdpSocket::bind("127.0.0.1:4243")?;

    let stdin = io::stdin();
    for line in stdin.lock().lines() {
        let line = line.unwrap();
        println!("Line read from stdin '{}'", line);
        if &line == "BYE" {
            break;
        }

        socket
            .send_to(line.as_bytes(), "127.0.0.1:4242")
            .expect("Error on send");

        let mut buf = [0; 2048];
        let (amt, _src) = socket.recv_from(&mut buf)?;

        let echo = str::from_utf8(&buf[..amt]).unwrap();
        println!("Echo {}", echo);
    }
    Ok(())
}
