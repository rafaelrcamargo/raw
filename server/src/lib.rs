use shared::{utils, Transaction};
use smallvec::SmallVec;

/// Static HTTP 404 response
pub const NOT_FOUND: &[u8] = b"HTTP/1.1 404 Not Found\r\nContent-Length: 0\r\n\r\n";
/// Static HTTP 522 response
pub const UNPROCESSABLE_ENTITY: &[u8] = b"HTTP/1.1 422 Unprocessable Entity\r\nContent-Length: 0\r\n\r\n";

/// Find a pattern in a slice
pub fn find_subsequence(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    haystack
        .windows(needle.len())
        .position(|window| window == needle)
}

/// Convert the transactions to a JSON string
pub fn to_json(transactions: SmallVec<[Transaction; 10]>) -> String {
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
