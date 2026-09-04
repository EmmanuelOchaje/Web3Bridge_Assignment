use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};

#[tokio::main]
async fn main() {
    let listener = TcpListener::bind("127.0.0.1:8080")
        .await
        .expect("Failed to bind to port 8080");

    println!("Restaurant server listening on 127.0.0.1:8080");

    loop {
        let (stream, addr) = listener
            .accept()
            .await
            .expect("Failed to accept connection");

        println!("New connection from {}", addr);

        tokio::spawn(async move {
            if let Err(e) = handle_connection(stream).await {
                eprintln!("Error handling connection: {}", e);
            }
        });
    }
}

async fn handle_connection(mut stream: TcpStream) -> tokio::io::Result<()> {
    let mut buffer = vec![0; 1024];

    let n = stream.read(&mut buffer).await?;

    let request = String::from_utf8_lossy(&buffer[..n]);

    println!("Request:\n{}\n", request);

    let request_line = request.lines().next().unwrap_or("");
    let parts: Vec<&str> = request_line.split_whitespace().collect();

    let (status_line, response_body, content_type) = if parts.len() >= 2 {
        let method = parts[0];
        let route = parts[1];
        match (method, route) {
            ("GET", "/") => {
                (
                    "HTTP/1.1 200 OK",
                    r#"<!DOCTYPE html>
<html>
<head>
    <title>Restaurant Order System</title>
    <style>
        body { font-family: Arial, sans-serif; margin: 40px; background-color: #f5f5f5; }
        .container { max-width: 600px; margin: 0 auto; background: white; padding: 20px; border-radius: 8px; }
        h1 { color: #333; }
        .menu { margin: 20px 0; }
        .menu-item { padding: 10px; background: #f9f9f9; margin: 10px 0; border-left: 4px solid #ff6b6b; }
        input { padding: 8px; margin: 5px; width: 200px; }
        button { padding: 10px 20px; background-color: #ff6b6b; color: white; border: none; border-radius: 4px; cursor: pointer; }
        button:hover { background-color: #ff5252; }
    </style>
</head>
<body>
    <div class="container">
        <h1>Welcome to Our Restaurant!</h1>
        <h2>Available Menu:</h2>
        <div class="menu">
            <div class="menu-item">Jollof Rice</div>
            <div class="menu-item">Fried Rice</div>
            <div class="menu-item">Chicken</div>
            <div class="menu-item">Burger</div>
        </div>
        <h2>Place Your Order:</h2>
        <form action="http://127.0.0.1:8080/order" method="POST">
            <div>
                <label>Food Item:</label><br>
                <input type="text" name="food" placeholder="e.g., Jollof Rice" required>
            </div>
            <div>
                <label>Quantity:</label><br>
                <input type="number" name="quantity" min="1" placeholder="e.g., 2" required>
            </div>
            <button type="submit">Order Now</button>
        </form>
        <hr>
        <p><strong>API Routes:</strong></p>
        <ul>
            <li>GET /menu - Get menu in JSON</li>
            <li>POST /order - Submit order</li>
        </ul>
    </div>
</body>
</html>"#,
                    "text/html",
                )
            }
            ("GET", "/menu") => {
                (
                    "HTTP/1.1 200 OK",
                    r#"{"foods": ["Jollof Rice", "Fried Rice", "Chicken", "Burger"]}"#,
                    "application/json",
                )
            }
            ("POST", "/order") => {
                if let Some(body_start) = request.find("\r\n\r\n") {
                    let body = &request[body_start + 4..];
                    if !body.is_empty() {
                        println!("Order details: {}", body);
                    }
                }
                (
                    "HTTP/1.1 200 OK",
                    r#"{"message": "Order received successfully"}"#,
                    "application/json",
                )
            }
            _ => (
                "HTTP/1.1 404 Not Found",
                r#"{"error": "Route not found"}"#,
                "application/json",
            ),
        }
    } else {
        (
            "HTTP/1.1 400 Bad Request",
            r#"{"error": "Invalid request"}"#,
            "application/json",
        )
    };

    let content_length = response_body.len();
    let response = format!(
        "{}\r\nContent-Type: {}\r\nContent-Length: {}\r\n\r\n{}",
        status_line, content_type, content_length, response_body
    );

    stream.write_all(response.as_bytes()).await?;

    stream.flush().await?;

    println!("Response sent\n");

    Ok(())
}
