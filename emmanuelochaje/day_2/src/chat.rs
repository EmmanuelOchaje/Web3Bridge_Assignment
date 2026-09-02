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
        let (stream, address) = listener.accept().await.unwrap();

        println!("{} connected", address);

        let tx = tx.clone();

        tokio::spawn(async move {
            handle_user(stream, address.to_string(), tx).await;
        });
    }
}

async fn handle_user(
    stream: TcpStream,
    name: String,
    tx: broadcast::Sender<String>,
) {
    let mut rx = tx.subscribe();

    let (reader, mut writer) = stream.into_split();

    let mut reader = BufReader::new(reader);

    let mut line = String::new();

    loop {
        tokio::select! {
            result = reader.read_line(&mut line) => {
                match result {
                    Ok(0) => {
                        println!("{} disconnected", name);
                        break;
                    }

                    Ok(_) => {
                        let timestamp = chrono::Local::now().format("%H:%M:%S");

                        let message = format!(
                            "[{}] user name {}:  message:{}",
                            timestamp, name, line
                        );

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
}