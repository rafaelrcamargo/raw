use std::{
    io::prelude::*,
    net::{TcpListener, UdpSocket},
};

use shared::{utils, IncomingTransaction, SuccessfulTransaction, Transaction, SIZE};
use smallvec::SmallVec;

const NOT_FOUND: &[u8] = b"HTTP/1.1 404 Not Found\r\nContent-Length: 0\r\n\r\n";
const UNPROCESSABLE_ENTITY: &[u8] =
    b"HTTP/1.1 422 Unprocessable Entity\r\nContent-Length: 0\r\n\r\n";

fn find_subsequence(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    haystack
        .windows(needle.len())
        .position(|window| window == needle)
}

fn main() {
    let tcp_port = std::env::var("TCP_PORT").unwrap_or("9999".to_string());
    let listener = TcpListener::bind(format!("0.0.0.0:{}", tcp_port)).unwrap();
    let socket = UdpSocket::bind("0.0.0.0:4040").unwrap();

    println!("Server started! (TCP: {}, UDP: {})", tcp_port, udp_port);

    for stream in listener.incoming() {
        // let before = Instant::now();

        let mut stream = stream.unwrap();
        stream.set_nodelay(true).unwrap();

        let mut buf = [0; 224]; // That's the exact size of a request
        let end = stream.read(&mut buf).unwrap(); // Do I need to say this is unsafe?

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
                        /* println!(
                            "Unprocessable entity: {:?} {:?}",
                            e,
                            String::from_utf8_lossy(body)
                        ); */
                        // println!("Unprocessable entity: {:.2?}", before.elapsed());
                        continue;
                    }
                };
                // println!("Parsed JSON: {:.2?}", before.elapsed());

                socket
                    .send_to(
                        &[&[id], bincode::serialize(&body).unwrap().as_slice()].concat(),
                        "database:4242",
                    )
                    .expect("Error on send");
                // println!("DB Req: {:.2?}", before.elapsed());

                let mut buf = [0; 8];
                let amt = socket.recv(&mut buf).unwrap();
                // println!("DB Resp: {} - {:.2?}", amt, before.elapsed());

                if amt == 0 {
                    stream.write_all(UNPROCESSABLE_ENTITY).unwrap();
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

                socket
                    .send_to(&[id], "database:4242")
                    .expect("Error on send");

                let mut buf = [0; (SIZE as usize) * 10];
                let amt = socket.recv(&mut buf).unwrap();

                let transactions = to_json(
                    buf[..amt]
                        .chunks(SIZE as usize)
                        .map(|x| bincode::deserialize::<Transaction>(x).unwrap())
                        .collect::<SmallVec<[Transaction; 10]>>(),
                );

                // println!("Ops. GET: {} - {:.2?}",transactions.len(),before.elapsed());

                let mut resp = String::with_capacity(768);
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

fn to_json(transactions: SmallVec<[Transaction; 10]>) -> String {
    let last = transactions.last().unwrap();
    let mut resp = String::with_capacity(704);

    resp.push_str(r#"{"saldo":{"total":"#);
    resp.push_str(&last.balance.to_string());
    resp.push_str(r#","data_extrato":"#);
    resp.push_str(&utils::get_time().to_string());
    resp.push_str(r#","limite":"#);
    resp.push_str(&last.limit.to_string());
    resp.push_str(r#"},"ultimas_transacoes":["#);

    transactions.iter().rev().for_each(|x| {
        resp.push_str(r#"{"valor":"#);
        resp.push_str(&x.value.to_string());
        resp.push_str(r#","tipo":""#);
        resp.push_str(&(x.operation as char).to_string());
        resp.push_str(r#"","descricao":""#);
        resp.push_str(&String::from_utf8_lossy(&x.description).replace('\0', ""));
        resp.push_str(r#"","realizada_em":"#);
        resp.push_str(&x.timestamp.to_string());
        resp.push_str(r#"},"#);
    });

    resp.pop(); // Remove the last comma
    resp.push_str("]}");

    resp
}
