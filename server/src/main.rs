#![feature(bufread_skip_until)]

use std::{
    io::prelude::*,
    net::{TcpListener, UdpSocket},
};

use shared::{utils, NewTransaction, SuccessfulTransaction, Transaction, SIZE};

const NOT_FOUND: &[u8] = b"HTTP/1.1 404 Not Found\r\nContent-Length: 0\r\n\r\n";
const _UNPROCESSABLE_ENTITY: &[u8] =
    b"HTTP/1.1 422 Unprocessable Entity\r\nContent-Length: 0\r\n\r\n";

fn find_subsequence(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    haystack
        .windows(needle.len())
        .position(|window| window == needle)
}

fn main() {
    let listener = TcpListener::bind("127.0.0.1:7878").unwrap();
    let socket = UdpSocket::bind("127.0.0.1:4243").unwrap();

    dbg!("Server started");
    for stream in listener.incoming() {
        let mut stream = stream.unwrap();
        let mut buf = [0; 256];
        stream.read(&mut buf).unwrap();

        match buf[4] {
            32 => {
                let id = buf[15] - b'0';

                if id > 5 {
                    stream.write_all(NOT_FOUND).unwrap();
                    continue;
                }

                let body = &buf[find_subsequence(&buf, b"\r\n\r\n").unwrap()
                    ..find_subsequence(&buf, &[0]).unwrap()];
                let body = serde_json::from_slice::<NewTransaction>(body).unwrap();

                dbg!(bincode::serialize(&body).unwrap().len());

                socket
                    .send_to(
                        &[&[id], bincode::serialize(&body).unwrap().as_slice()].concat(),
                        "127.0.0.1:4242",
                    )
                    .expect("Error on send");

                dbg!("Sent");

                let mut buf = [0; (SIZE as usize) * 10 + 1];
                let amt = socket.recv(&mut buf).unwrap();

                dbg!(amt);

                let resp = bincode::deserialize::<SuccessfulTransaction>(&buf[..amt]).unwrap();
                let json = serde_json::to_string(&resp).unwrap();

                dbg!(&json);

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

                continue; // We don't want to close the stream
            }
            47 => {
                let id = buf[14] - b'0';
                dbg!(id);
                /* socket
                    .send_to(&bincode::serialize(&id).unwrap(), "127.0.0.1:4242")
                    .expect("Error on send");

                dbg!("Sent");

                let mut buf = [0; (SIZE as usize) * 10 + 1];
                let amt = socket.recv(&mut buf).unwrap();

                let transactions = buf[..amt]
                    .chunks(SIZE as usize)
                    .map(|x| bincode::deserialize::<Transaction>(x).unwrap());

                let resp = to_json(transactions.collect::<Vec<Transaction>>());
                dbg!(&resp);
                /* let resp = bincode::deserialize::<Vec<Transaction>>(&buf[..amt]).unwrap();
                let json = serde_json::to_string(&resp).unwrap();
                dbg!(&json); */

                stream
                    .write_all(
                        format!(
                            "HTTP/1.1 200 OK\r\nContent-Length: {}\r\n\r\n{}",
                            resp.len(),
                            resp
                        )
                        .as_bytes(),
                    )
                    .unwrap(); */

                continue; // We don't want to close the stream
            }
            _ => {
                stream.write_all(NOT_FOUND).unwrap();
                continue; // We don't want to close the stream
            }
        }
    }
}

/**
 * We need now to impl for Transaction to map to JSON
 * the format should be:
{
  "saldo": {
    "total": -9098,
    "data_extrato": "2024-01-17T02:34:41.217753Z",
    "limite": 100000
  },
  "ultimas_transacoes": [
    {
      "valor": 10,
      "tipo": "c",
      "descricao": "descricao",
      "realizada_em": "2024-01-17T02:34:38.543030Z"
    },
    {
      "valor": 90000,
      "tipo": "d",
      "descricao": "descricao",
      "realizada_em": "2024-01-17T02:34:38.543030Z"
    }
  ]
}
 * Where saldo is the balance, limit and the date of the last transaction
 * and ultimas_transacoes is the last 10 transactions
 */
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
        .map(|x| {
            dbg!(String::from_utf8_lossy(&x.description));

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
        r#"{{{}"ultimas_transacoes":[{}]}}"#,
        saldo, ultimas_transacoes
    )
}
