#[macro_use]extern crate tokio;
use tokio::net::{TcpStream, TcpListener};
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};

async fn process_socket(mut stream: TcpStream) -> Result<(), Box<dyn std::error::Error>> {
    let mut data = String::new();
    stream.read_to_string(&mut data).await?;
    println!("data: {}", data);
    stream.write_all(b"Connection: close");
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>>{
    let listener = TcpListener::bind("127.0.0.1:8080").await?;

    loop {
        let (stream, addr) = listener.accept().await?;
        println!("new connection from: {}", addr);
        process_socket(stream).await?;
    }
}
