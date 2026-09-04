use chrono::Local;
use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
    net::{TcpListener, TcpStream},
    sync::broadcast,
};

#[tokio::main]
async fn main() {
    let listener = TcpListener::bind("127.0.0.1:8080")
        .await
        .unwrap();

    println!("Chat server running on 127.0.0.1:8080");

    let (tx, _) = broadcast::channel::<String>(100);

    loop {
        let (stream, _address) = listener.accept().await.unwrap();

        let tx = tx.clone();

        tokio::spawn(async move {
            handle_user(stream, tx).await;
        });
    }
}

fn get_timestamp() -> String {
    Local::now().format("%H:%M:%S").to_string()
}

async fn handle_user(stream: TcpStream, tx: broadcast::Sender<String>) {
    let mut rx = tx.subscribe();
    let (reader, mut writer) = stream.into_split();
    let mut reader = BufReader::new(reader);

    // Ask for username
    let _ = writer.write_all(b"Enter your username:\n").await;

    let mut username_line = String::new();
    if reader.read_line(&mut username_line).await.is_err() {
        return;
    }

    let username = username_line.trim().to_string();
    if username.is_empty() {
        return;
    }

    // Broadcast join notification
    let timestamp = get_timestamp();
    let join_message = format!("[{}] {} joined the chat\n", timestamp, username);
    let _ = tx.send(join_message);

    println!("[{}] {} connected", timestamp, username);

    let mut line = String::new();

    loop {
        tokio::select! {
            result = reader.read_line(&mut line) => {
                match result {
                    Ok(0) => {
                        // Connection closed
                        let timestamp = get_timestamp();
                        let leave_message = format!("[{}] {} left the chat\n", timestamp, username);
                        let _ = tx.send(leave_message);
                        println!("[{}] {} disconnected", timestamp, username);
                        break;
                    }

                    Ok(_) => {
                        let trimmed = line.trim();

                        if trimmed == "/quit" {
                            // User quit gracefully
                            let timestamp = get_timestamp();
                            let leave_message = format!("[{}] {} left the chat\n", timestamp, username);
                            let _ = tx.send(leave_message);
                            println!("[{}] {} quit", timestamp, username);
                            break;
                        } else if !trimmed.is_empty() {
                            // Send regular chat message
                            let timestamp = get_timestamp();
                            let message = format!("[{}] {}: {}\n", timestamp, username, trimmed);
                            let _ = tx.send(message);
                        }

                        line.clear();
                    }

                    Err(_) => {
                        break;
                    }
                }
            }

            result = rx.recv() => {
                match result {
                    Ok(message) => {
                        if writer
                            .write_all(message.as_bytes())
                            .await
                            .is_err()
                        {
                            break;
                        }
                    }

                    Err(_) => {
                        break;
                    }
                }
            }
        }
    }
}