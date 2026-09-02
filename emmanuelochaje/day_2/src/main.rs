use chrono::Local;
use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
    net::{TcpListener, TcpStream},
    sync::broadcast,
};

type Msg = (String, String);

#[tokio::main]
async fn main() {
    let listener = TcpListener::bind("127.0.0.1:8080").await.unwrap();

    println!("Chat server running on 127.0.0.1:8080");

    let (tx, _) = broadcast::channel::<Msg>(100);

    loop {
        let (stream, address) = listener.accept().await.unwrap();
        let tx = tx.clone();

        tokio::spawn(async move {
            handle_user(stream, address.to_string(), tx).await;
        });
    }
}

fn timestamp() -> String {
    Local::now().format("%H:%M:%S").to_string()
}

async fn handle_user(stream: TcpStream, id: String, tx: broadcast::Sender<Msg>) {
    let mut rx = tx.subscribe();

    let (reader, mut writer) = stream.into_split();
    let mut reader = BufReader::new(reader);

    if writer.write_all(b"Enter your username:\n").await.is_err() {
        return;
    }

    let mut username = String::new();
    match reader.read_line(&mut username).await {
        Ok(0) | Err(_) => return,
        Ok(_) => {}
    }
    let username = username.trim().to_string();
    if username.is_empty() {
        return;
    }

    let join_message = format!("[{}] {} joined the chat\n", timestamp(), username);
    let _ = tx.send((id.clone(), join_message));

    let mut line = String::new();

    loop {
        tokio::select! {
            result = reader.read_line(&mut line) => {
                match result {
                    Ok(0) => break,
                    Ok(_) => {
                        let text = line.trim_end();

                        if text == "/quit" {
                            line.clear();
                            break;
                        }

                        let message = format!("[{}] {}: {}\n", timestamp(), username, text);
                        let _ = tx.send((id.clone(), message));
                        line.clear();
                    }
                    Err(_) => break,
                }
            }

            result = rx.recv() => {
                match result {
                    Ok((sender_id, message)) => {
                        if sender_id != id
                            && writer.write_all(message.as_bytes()).await.is_err()
                        {
                            break;
                        }
                    }
                    Err(_) => break,
                }
            }
        }
    }

    let leave_message = format!("[{}] {} left the chat\n", timestamp(), username);
    let _ = tx.send((id, leave_message));
}
