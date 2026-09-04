use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
    net::{TcpListener, TcpStream},
    sync::broadcast,
};
use std::time::SystemTime;

#[tokio::main]
async fn main() {
    let listener = TcpListener::bind("127.0.0.1:8080")
        .await
        .expect("Failed to bind to port 8080");

    println!("Chat server running on 127.0.0.1:8080");
    println!("Waiting for clients...\n");

    let (tx, _) = broadcast::channel::<String>(100);

    loop {
        let (stream, addr) = listener.accept().await.unwrap();

        println!("Connection attempt from {}", addr);

        let tx = tx.clone();

        tokio::spawn(async move {
            handle_user(stream, tx).await;
        });
    }
}

async fn handle_user(stream: TcpStream, tx: broadcast::Sender<String>) {
    let mut rx = tx.subscribe();

    let (reader, mut writer) = stream.into_split();
    let mut reader = BufReader::new(reader);

    if writer
        .write_all(b"Enter your username: ")
        .await
        .is_err()
    {
        eprintln!("Failed to prompt for username");
        return;
    }

    let mut username = String::new();

    if reader.read_line(&mut username).await.is_err() {
        eprintln!("Failed to read username");
        return;
    }

    let username = username.trim().to_string();

    if username.is_empty() {
        let _ = writer.write_all(b"Username cannot be empty\n").await;
        return;
    }

    println!("User '{}' joined", username);

    let join_message = format!(
        "[{}] {} joined the chat\n",
        get_timestamp(),
        username
    );
    let _ = tx.send(join_message);

    let mut line = String::new();

    loop {
        line.clear();

        tokio::select! {
            result = reader.read_line(&mut line) => {
                match result {
                    Ok(0) => {
                        println!("User '{}' disconnected", username);

                        let leave_message = format!(
                            "[{}] {} left the chat\n",
                            get_timestamp(),
                            username
                        );
                        let _ = tx.send(leave_message);

                        break;
                    }
                    Ok(_) => {
                        let message = line.trim();

                        if message == "/quit" {
                            println!("User '{}' quit gracefully", username);

                            let leave_message = format!(
                                "[{}] {} left the chat\n",
                                get_timestamp(),
                                username
                            );
                            let _ = tx.send(leave_message);

                            break;
                        }
                        let formatted_message = format!(
                            "[{}] {}: {}\n",
                            get_timestamp(),
                            username,
                            message
                        );

                        println!("{}: {}", username, message);

                        let _ = tx.send(formatted_message);
                    }

                    Err(_) => {
                        println!("Error reading from user '{}'", username);

                        let leave_message = format!(
                            "[{}] {} left the chat\n",
                            get_timestamp(),
                            username
                        );
                        let _ = tx.send(leave_message);

                        break;
                    }
                }
            }

            result = rx.recv() => {
                match result {
                    Ok(message) => {
                        if writer.write_all(message.as_bytes()).await.is_err() {
                            println!("Failed to send message to user '{}'", username);
                            break;
                        }
                    }
                    Err(_) => {
                        println!("Broadcast channel error for user '{}'", username);
                        break;
                    }
                }
            }
        }
    }
}

fn get_timestamp() -> String {
    use std::time::UNIX_EPOCH;

    let now = SystemTime::now();
    let duration = now
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();

    let total_seconds = duration.as_secs() % 86400;

    let hours = total_seconds / 3600;
    let minutes = (total_seconds % 3600) / 60;
    let seconds = total_seconds % 60;

    format!("{:02}:{:02}:{:02}", hours, minutes, seconds)
}
