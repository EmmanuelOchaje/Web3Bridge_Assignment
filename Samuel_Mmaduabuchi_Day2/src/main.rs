use std::{
    io,
    sync::atomic::{AtomicU64, Ordering},
};

use chrono::Local;
use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
    net::{TcpListener, TcpStream},
    sync::broadcast,
};

const ADDRESS: &str = "127.0.0.1:8080";
static NEXT_USER_ID: AtomicU64 = AtomicU64::new(1);

#[derive(Clone)]
struct ChatMessage {
    sender_id: u64,
    text: String,
}

#[tokio::main]
async fn main() -> io::Result<()> {
    let listener = TcpListener::bind(ADDRESS).await?;
    let (tx, _) = broadcast::channel::<ChatMessage>(100);

    println!("Chat server running on {ADDRESS}");

    loop {
        let (stream, address) = listener.accept().await?;
        let tx = tx.clone();

        tokio::spawn(async move {
            if let Err(error) = handle_user(stream, tx).await {
                eprintln!("Error handling {address}: {error}");
            }
        });
    }
}

async fn handle_user(stream: TcpStream, tx: broadcast::Sender<ChatMessage>) -> io::Result<()> {
    let user_id = NEXT_USER_ID.fetch_add(1, Ordering::Relaxed);
    let (reader, mut writer) = stream.into_split();
    let mut reader = BufReader::new(reader);

    let Some(username) = read_username(&mut reader, &mut writer).await? else {
        return Ok(());
    };

    let mut rx = tx.subscribe();
    broadcast(&tx, user_id, format!("{} joined the chat", username));

    let mut line = String::new();

    loop {
        tokio::select! {
            result = reader.read_line(&mut line) => {
                match result {
                    Ok(0) => break,
                    Ok(_) => {
                        let message = line.trim_end_matches(['\r', '\n']);

                        if message == "/quit" {
                            break;
                        }

                        if !message.is_empty() {
                            broadcast(&tx, user_id, format!("{username}: {message}"));
                        }

                        line.clear();
                    }
                    Err(error) => {
                        eprintln!("Could not read from {username}: {error}");
                        break;
                    }
                }
            }
            result = rx.recv() => {
                match result {
                    Ok(message) if message.sender_id != user_id => {
                        if writer.write_all(message.text.as_bytes()).await.is_err() {
                            break;
                        }
                    }
                    Ok(_) | Err(broadcast::error::RecvError::Lagged(_)) => {}
                    Err(broadcast::error::RecvError::Closed) => break,
                }
            }
        }
    }

    broadcast(&tx, user_id, format!("{} left the chat", username));
    Ok(())
}

async fn read_username<R, W>(reader: &mut R, writer: &mut W) -> io::Result<Option<String>>
where
    R: AsyncBufReadExt + Unpin,
    W: AsyncWriteExt + Unpin,
{
    loop {
        writer.write_all(b"Enter your username:\n").await?;

        let mut username = String::new();
        if reader.read_line(&mut username).await? == 0 {
            return Ok(None);
        }

        let username = username.trim().to_owned();
        if !username.is_empty() {
            return Ok(Some(username));
        }
    }
}

fn broadcast(tx: &broadcast::Sender<ChatMessage>, sender_id: u64, message: String) {
    let text = format!("[{}] {message}\n", Local::now().format("%H:%M:%S"));
    let _ = tx.send(ChatMessage { sender_id, text });
}
