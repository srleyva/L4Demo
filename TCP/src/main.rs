use rustyline::error::ReadlineError;
use rustyline::Editor;
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
                    socket.write_all(b"message recieved").await.unwrap();
                }
                Err(e) => println!("couldn't get client: {:?}", e),
            }
        }
    });

    enum Command {
        Send(String, String),
        Err(String),
    }

    impl From<String> for Command {
        fn from(input: String) -> Self {
            let split_input: Vec<&str> = input.split(" ").collect();
            let command = split_input[0].to_owned();
            let address = split_input[1].to_owned();
            let message = split_input[1..].join(" ").to_owned();

            return match command.to_lowercase().as_str() {
                "send" => Self::Send(address, message),
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
                    let mut connection = TcpStream::connect("127.0.0.1:5050").await?;
                    connection.write_all(message.as_bytes()).await?;
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
