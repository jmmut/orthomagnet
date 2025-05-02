use std::io::{Read, Write};
use std::net::TcpStream;

fn main() -> std::io::Result<()> {
    let mut stream = TcpStream::connect("127.0.0.1:31415")?;

    stream.write(&[42])?;
    let mut buf = [0; 128];
    stream.read(&mut buf)?;
    println!("read: {:?}", buf);
    Ok(())
}
