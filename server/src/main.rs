use std::{
    io::prelude::*,
    net::{TcpListener, UdpSocket},
};

use shared::{utils, IncomingTransaction, SuccessfulTransaction, Transaction, SIZE};

const NOT_FOUND: &[u8] = b"HTTP/1.1 404 Not Found\r\nContent-Length: 0\r\n\r\n";
const UNPROCESSABLE_ENTITY: &[u8] =
    b"HTTP/1.1 422 Unprocessable Entity\r\nContent-Length: 0\r\n\r\n";

fn find_subsequence(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    haystack
        .windows(needle.len())
        .position(|window| window == needle)
}

fn main() {
    let tcp_port = std::env::var("TCP_PORT").unwrap_or("8080".to_string());
    let listener = TcpListener::bind(format!("0.0.0.0:{}", tcp_port)).unwrap();

    let udp_port = std::env::var("UDP_PORT").unwrap_or("8080".to_string());
    let socket = UdpSocket::bind(format!("0.0.0.0:{}", udp_port)).unwrap();

    println!("Server started on: {}", tcp_port);
    for stream in listener.incoming() {
        // let before = Instant::now();

        let mut buf = [0; 320]; // Stream buffer
        let mut stream = stream.unwrap();
        stream.read(&mut buf).unwrap();

        match buf[4] {
            32 => {
                let id = buf[15] - b'0'; // This is probably the unsafest safe rust code possible

                if id > 5 {
                    stream.write_all(NOT_FOUND).unwrap();
                    // println!("Invalid ID: {:.2?}", before.elapsed());
                    continue;
                }

                // This is as risky as it gets
                let start = find_subsequence(&buf, b"\r\n\r\n").unwrap() + 4;
                let end = find_subsequence(&buf, &[0]).unwrap();
                let body = &mut buf[start..end];

                let body = match simd_json::from_slice::<IncomingTransaction>(body) {
                    Ok(x) => {
                        if x.description.is_empty()
                            || x.description.len() > 10
                            || (x.kind != b'd' && x.kind != b'c')
                        {
                            stream.write_all(UNPROCESSABLE_ENTITY).unwrap();
                            // println!("Unprocessable entity: {:.2?}", before.elapsed());
                            continue;
                        }

                        x
                    }
                    Err(_) => {
                        stream.write_all(UNPROCESSABLE_ENTITY).unwrap();
                        // println!("Unprocessable entity: {:.2?}", before.elapsed());
                        continue;
                    }
                };

                socket
                    .send_to(
                        &[&[id], bincode::serialize(&body).unwrap().as_slice()].concat(),
                        "127.0.0.1:4242",
                    )
                    .expect("Error on send");

                // dbg!("Sent");

                let mut buf = [0; (SIZE as usize) * 10 + 1];
                let amt = socket.recv(&mut buf).unwrap();

                if amt == 0 {
                    stream.write_all(UNPROCESSABLE_ENTITY).unwrap();
                    // println!("Unprocessable entity: {:.2?}", before.elapsed());
                    continue;
                }

                let resp = bincode::deserialize::<SuccessfulTransaction>(&buf[..amt]).unwrap();
                let json = simd_json::to_string(&resp).unwrap();

                // println!("Ops. POST: {:.2?}", before.elapsed());
                stream
                    .write_all(
                        format!(
                            "HTTP/1.1 200 OK\r\nContent-Length: {}\r\n\r\n{}",
                            json.len(),
                            json
                        )
                        .as_bytes(),
                    )
                    .unwrap();

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

                socket
                    .send_to(&bincode::serialize(&id).unwrap(), "127.0.0.1:4242")
                    .expect("Error on send");

                // dbg!("Sent");

                let mut buf = [0; (SIZE as usize) * 10 + 1];
                let amt = socket.recv(&mut buf).unwrap();

                let transactions = buf[..amt]
                    .chunks(SIZE as usize)
                    .map(|x| bincode::deserialize::<Transaction>(x).unwrap());

                let resp = to_json(transactions.collect::<Vec<Transaction>>());

                /* let resp = bincode::deserialize::<Vec<Transaction>>(&buf[..amt]).unwrap();
                let json = simd_json::to_string(&resp).unwrap();
                dbg!(&json); */

                // println!("Ops. GET: {:.2?}", before.elapsed());
                stream
                    .write_all(
                        format!(
                            "HTTP/1.1 200 OK\r\nContent-Length: {}\r\n\r\n{}",
                            resp.len(),
                            resp
                        )
                        .as_bytes(),
                    )
                    .unwrap();

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

fn to_json(transactions: Vec<Transaction>) -> String {
    let last = transactions.last().unwrap();

    let saldo = format!(
        r#""saldo": {{"total":{},"data_extrato":"{}","limite":{}}}"#,
        last.balance,
        utils::get_time(),
        last.limit
    );

    let ultimas_transacoes = transactions
        .iter()
        .rev()
        .map(|x| {
            // dbg!(String::from_utf8_lossy(&x.description));

            format!(
                r#"{{"valor":{},"tipo":"{}","descricao":"{}","realizada_em":"{}"}}"#,
                x.value,
                (x.operation as char).to_string().replace('\0', ""),
                String::from_utf8_lossy(&x.description).replace('\0', ""),
                x.timestamp
            )
        })
        .collect::<Vec<String>>()
        .join(",");

    format!(
        r#"{{{},"ultimas_transacoes":[{}]}}"#,
        saldo, ultimas_transacoes
    )
}
