use rustyline::error::ReadlineError;
use rustyline::Editor;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::UdpSocket;

#[tokio::main]
async fn main() -> tokio::io::Result<()> {
    let socket = UdpSocket::bind("0.0.0.0:5051").await?;

    tokio::spawn(async move {
        println!("Listening for UDP on {}", socket.local_addr().unwrap());
        loop {
            let mut buf = [0; 10];
            let (amt, src) = socket.recv_from(&mut buf).await.expect("could not read");
            println!("Recved from {}", src);
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
                    let sender = UdpSocket::bind("127.0.0.1:0").await?;
                    sender.connect(address).await?;
                    sender.send(message.as_bytes()).await?;
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
