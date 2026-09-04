use std::net::SocketAddr;
use std::time::{SystemTime, UNIX_EPOCH};

use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::broadcast;

const ADDR: &str = "127.0.0.1:8080";

#[derive(Clone)]
struct ChatMessage {
    from: SocketAddr,
    text: String,
}

#[tokio::main]
async fn main() {
    let listener = TcpListener::bind(ADDR)
        .await
        .expect("failed to bind to 127.0.0.1:8080");

    println!("Chat server running on {ADDR}");

    let (tx, _rx) = broadcast::channel::<ChatMessage>(100);

    loop {
        let (stream, addr) = match listener.accept().await {
            Ok(pair) => pair,
            Err(e) => {
                eprintln!("failed to accept connection: {e}");
                continue;
            }
        };

        println!("{addr} connected");

        let tx = tx.clone();
        tokio::spawn(async move {
            if let Err(e) = handle_user(stream, addr, tx).await {
                eprintln!("{addr} ended with error: {e}");
            }
        });
    }
}

async fn handle_user(
    stream: TcpStream,
    addr: SocketAddr,
    tx: broadcast::Sender<ChatMessage>,
) -> std::io::Result<()> {
    let mut rx = tx.subscribe();

    let (reader, mut writer) = stream.into_split();
    let mut reader = BufReader::new(reader);
    let mut line = String::new();
    writer.write_all(b"Enter your username:\n").await?;
    let n = reader.read_line(&mut line).await?;
    if n == 0 {
        return Ok(());
    }
    let username = line.trim().to_string();
    let username = if username.is_empty() {
        addr.to_string()
    } else {
        username
    };
    line.clear();


    let _ = tx.send(ChatMessage {
        from: addr,
        text: format!("[{}] {username} joined the chat\n", timestamp()),
    });

    loop {
        tokio::select! {
            result = reader.read_line(&mut line) => {
                match result {
                    Ok(0) => break,
                    Ok(_) => {
                        let msg = line.trim_end().to_string();
                        line.clear();
                        if msg == "/quit" {
                            break;
                        }
                        if msg.is_empty() {
                            continue;
                        }
                        let _ = tx.send(ChatMessage {
                            from: addr,
                            text: format!("[{}] {username}: {msg}\n", timestamp()),
                        });
                    }
                    Err(_) => break,
                }
            }

            result = rx.recv() => {
                match result {
                    Ok(chat) => {
                        if chat.from == addr {
                            continue;
                        }
                        if writer.write_all(chat.text.as_bytes()).await.is_err() {
                            break;
                        }
                    }
                    Err(broadcast::error::RecvError::Lagged(_)) => continue,
                    Err(broadcast::error::RecvError::Closed) => break,
                }
            }
        }
    }


    println!("{addr} ({username}) disconnected");
    let _ = tx.send(ChatMessage {
        from: addr,
        text: format!("[{}] {username} left the chat\n", timestamp()),
    });

    Ok(())
}


fn timestamp() -> String {
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);

    let seconds_in_day = secs % 86_400;
    let hours = seconds_in_day / 3_600;
    let minutes = (seconds_in_day % 3_600) / 60;
    let seconds = seconds_in_day % 60;

    format!("{hours:02}:{minutes:02}:{seconds:02}")
}
