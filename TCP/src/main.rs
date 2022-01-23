use rustyline::error::ReadlineError;
use rustyline::Editor;
use tokio::fs::File;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};

#[tokio::main]
async fn main() -> tokio::io::Result<()> {
    let listener = TcpListener::bind("0.0.0.0:5050").await?;

    tokio::spawn(async move {
        println!("Listening on {}", listener.local_addr().unwrap());
        loop {
            match listener.accept().await {
                Ok((mut socket, addr)) => {
                    println!("new client: {:?}", addr);
                    let mut buffer = Vec::new();

                    let read = socket.read_to_end(&mut buffer).await.unwrap();
                    println!("read from client: {} bytes", read);
                    socket.write_all(b"message recieved").await.unwrap();
                }
                Err(e) => println!("couldn't get client: {:?}", e),
            }
        }
    });

    enum Command {
        Send(String, String),
        File(String, String),
        Err(String),
    }

    impl From<String> for Command {
        fn from(input: String) -> Self {
            let split_input: Vec<&str> = input.split(" ").collect();
            let command = split_input[0].to_owned();
            let address = split_input[1].to_owned();
            let message = split_input[2..].join(" ").to_owned();

            return match command.to_lowercase().as_str() {
                "send" => Self::Send(address, message),
                "send_file" => Self::File(address, message),
                _ => Self::Err(input),
            };
        }
    }

    let mut rl = Editor::<()>::new();
    loop {
        let readline = rl.readline(">> ");
        match readline {
            Ok(line) => match Command::from(line) {
                Command::Send(address, message) => {
                    let mut connection = TcpStream::connect(address).await?;
                    connection.write_all(message.as_bytes()).await?;
                }
                Command::File(address, file_path) => {
                    let mut file = File::open(file_path).await?;
                    let mut connection = TcpStream::connect(address).await?;
                    let mut buffer = Vec::new();
                    let read = file.read_to_end(&mut buffer).await?;
                    println!("Sending {} bytes to client", read);
                    connection.write_all(&mut buffer).await?;
                }
                Command::Err(input) => println!("did not recognize input"),
            },
            Err(ReadlineError::Interrupted) => {
                println!("CTRL-C");
                break;
            }
            Err(ReadlineError::Eof) => {
                println!("CTRL-D");
                break;
            }
            Err(err) => {
                println!("Error: {:?}", err);
                break;
            }
        }
    }

    Ok(())
}
