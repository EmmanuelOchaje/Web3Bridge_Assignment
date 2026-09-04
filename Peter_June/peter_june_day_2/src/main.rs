use chrono::Local;
use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
    net::{TcpListener, TcpStream},
    sync::broadcast,
};

#[tokio::main]
async fn main() {
    let listener = TcpListener::bind("127.0.0.1:8080").await.unwrap();

    println!("Chat server running on 127.0.0.1:8080");

    let (tx, _) = broadcast::channel::<String>(100);

    loop {
        let (stream, _) = listener.accept().await.unwrap();

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

    if writer.write_all(b"Enter your username: ").await.is_err() {
        return;
    }

    let mut username = String::new();

    let res = reader.read_line(&mut username).await;

    let username = username.trim().to_string();

    match res {
        Ok(0) => {
            return;
        }

        Ok(_) => {
            println!("{} connected", username);
            println!("Username: {}", username);
        }

        Err(_) => {
            return;
        }
    }

    let timestamp = current_time();

    let join_message = format!("[{timestamp}] {username} joined the chat\n");

    let _ = tx.send(join_message);

    let mut line = String::new();

    loop {
        tokio::select! {
            result = reader.read_line(&mut line) => {
                match result {
                    Ok(0) => {
                        println!("{} disconnected", username);
                        break;
                    }

                    Ok(_) => {
                        let message_text = line.trim();

                        if message_text == "/quit" {
                            break;
                        }

                        let timestamp = current_time();
                        let message =
                            format!("[{timestamp}] {username}: {message_text}\n");

                        let _ = tx.send(message);

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

    let timestamp = current_time();
    let leave_message = format!("[{timestamp}] {username} left the chat\n");
    let _ = tx.send(leave_message);
}

fn current_time() -> String {
    Local::now().format("%H:%M:%S").to_string()
}
