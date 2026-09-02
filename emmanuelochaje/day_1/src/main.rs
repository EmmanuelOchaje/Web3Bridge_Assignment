use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:8080").await?;
    println!("Server running on http://127.0.0.1:8080");

    loop {
        let (stream, _addr) = listener.accept().await?;

        tokio::spawn(async move {
            handle_connection(stream).await;
        });
    }
}

async fn handle_connection(mut stream: TcpStream) {
    let mut buffer = [0; 1024];

    let bytes_read = match stream.read(&mut buffer).await {
        Ok(n) => n,
        Err(_) => return,
    };

    let request = String::from_utf8_lossy(&buffer[..bytes_read]);

    let first_line = request.lines().next().unwrap_or("");
    let mut parts = first_line.split_whitespace();
    let method = parts.next().unwrap_or("");
    let path = parts.next().unwrap_or("");

    println!("Got request: {} {}", method, path);

    let response = if method == "GET" && path == "/menu" {
        let body = r#"{
  "foods": [
    "Jollof Rice",
    "Fried Rice",
    "Chicken",
    "Burger"
  ]
}"#;
        format!("HTTP/1.1 200 OK\r\nContent-Type: application/json\r\n\r\n{}", body)

    } else if method == "POST" && path == "/order" {
        if let Some(body) = request.split("\r\n\r\n").nth(1) {
            println!("New order received:");
            println!("{}", body.trim());
        }

        let body = r#"{
  "message": "Order received successfully"
}"#;
        format!("HTTP/1.1 200 OK\r\nContent-Type: application/json\r\n\r\n{}", body)

    } else {
        let body = r#"{
  "error": "Route not found"
}"#;
        format!("HTTP/1.1 404 Not Found\r\nContent-Type: application/json\r\n\r\n{}", body)
    };

    let _ = stream.write_all(response.as_bytes()).await;
}