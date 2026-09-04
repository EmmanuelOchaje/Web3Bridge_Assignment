

use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};

const ADDR: &str = "127.0.0.1:8080";

#[tokio::main]
async fn main() {
    let listener = TcpListener::bind(ADDR)
        .await
        .expect("failed to bind to 127.0.0.1:8080");

    println!("Restaurant server running on http://{ADDR}");
    loop {
        match listener.accept().await {
            Ok((stream, peer)) => {
                tokio::spawn(async move {
                    if let Err(e) = handle_connection(stream).await {
                        eprintln!("connection from {peer} ended with error: {e}");
                    }
                });
            }
            Err(e) => eprintln!("failed to accept connection: {e}"),
        }
    }
}


async fn handle_connection(mut stream: TcpStream) -> std::io::Result<()> {
    let request = read_request(&mut stream).await?;
    let request_line = request.lines().next().unwrap_or_default();
    let mut parts = request_line.split_whitespace();
    let method = parts.next().unwrap_or_default();
    let path = parts.next().unwrap_or_default();

    let response = match (method, path) {
        ("GET", "/menu") => {
            let body = r#"{
  "foods": [
    "Jollof Rice",
    "Fried Rice",
    "Chicken",
    "Burger"
  ]
}"#;
            http_response("200 OK", body)
        }
        ("POST", "/order") => {
            // Bonus: pull the body out of the request and print the order.
            if let Some(body) = request.split_once("\r\n\r\n").map(|(_, b)| b) {
                let food = json_string_field(body, "food");
                let quantity = json_number_field(body, "quantity");
                if let (Some(food), Some(quantity)) = (food, quantity) {
                    println!("\nNew order received:\nFood: {food}\nQuantity: {quantity}");
                } else if !body.trim().is_empty() {
                    println!("\nNew order received (raw body): {}", body.trim());
                }
            }

            let body = r#"{
  "message": "Order received successfully"
}"#;
            http_response("200 OK", body)
        }
        _ => {
            let body = r#"{
  "error": "Route not found"
}"#;
            http_response("404 Not Found", body)
        }
    };

    stream.write_all(response.as_bytes()).await?;
    stream.flush().await?;
    Ok(())
}

async fn read_request(stream: &mut TcpStream) -> std::io::Result<String> {
    let mut buf = vec![0u8; 4096];
    let mut data = Vec::new();

    loop {
        let n = stream.read(&mut buf).await?;
        if n == 0 {
            break;
        }
        data.extend_from_slice(&buf[..n]);
        if let Some(header_end) = find_subslice(&data, b"\r\n\r\n") {
            let headers = String::from_utf8_lossy(&data[..header_end]);
            let content_length = headers
                .lines()
                .find_map(|line| {
                    let (name, value) = line.split_once(':')?;
                    if name.trim().eq_ignore_ascii_case("content-length") {
                        value.trim().parse::<usize>().ok()
                    } else {
                        None
                    }
                })
                .unwrap_or(0);

            let body_so_far = data.len() - (header_end + 4);
            while data.len() - (header_end + 4) < content_length {
                let n = stream.read(&mut buf).await?;
                if n == 0 {
                    break;
                }
                data.extend_from_slice(&buf[..n]);
            }
            let _ = body_so_far;
            break;
        }
    }

    Ok(String::from_utf8_lossy(&data).into_owned())
}

fn http_response(status: &str, body: &str) -> String {
    format!(
        "HTTP/1.1 {status}\r\n\
         Content-Type: application/json\r\n\
         Content-Length: {len}\r\n\
         Connection: close\r\n\
         \r\n\
         {body}",
        len = body.len()
    )
}

fn find_subslice(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    haystack
        .windows(needle.len())
        .position(|window| window == needle)
}

fn json_string_field(body: &str, key: &str) -> Option<String> {
    let pattern = format!("\"{key}\"");
    let after_key = &body[body.find(&pattern)? + pattern.len()..];
    let after_colon = &after_key[after_key.find(':')? + 1..];
    let start = after_colon.find('"')? + 1;
    let rest = &after_colon[start..];
    let end = rest.find('"')?;
    Some(rest[..end].to_string())
}

fn json_number_field(body: &str, key: &str) -> Option<u64> {
    let pattern = format!("\"{key}\"");
    let after_key = &body[body.find(&pattern)? + pattern.len()..];
    let after_colon = &after_key[after_key.find(':')? + 1..];
    let digits: String = after_colon
        .trim_start()
        .chars()
        .take_while(|c| c.is_ascii_digit())
        .collect();
    digits.parse().ok()
}
