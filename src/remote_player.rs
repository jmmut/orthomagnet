use crate::AnyError;
use nanoserde::{DeBin, SerBin};
use std::io::{ErrorKind, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::mpsc::{Receiver, Sender};
use std::thread;
use std::time::Duration;

pub const PORT: u16 = 31415;

#[derive(SerBin, DeBin, Debug)]
pub enum Command {
    StoneHover { x: i32, y: i32 },
    StopStoneHover,
    Connected,
}

#[must_use]
pub fn serve() -> (Receiver<Command>, Sender<Command>) {
    let (to_local, from_client) = std::sync::mpsc::channel::<Command>();
    let (to_client, from_local) = std::sync::mpsc::channel::<Command>();
    thread::spawn(|| try_server_thread(to_local, from_local));
    (from_client, to_client)
}

fn try_server_thread(to_local: Sender<Command>, from_local: Receiver<Command>) {
    if let Err(e) = server_thread(to_local, from_local) {
        println!("Server thread error: {}", e);
    }
}

fn server_thread(to_local: Sender<Command>, from_local: Receiver<Command>) -> Result<(), AnyError> {
    let listener = TcpListener::bind(&format!("127.0.0.1:{}", PORT)).unwrap();
    println!("Server listening on port {}", PORT);
    loop {
        let (stream, _socket_addr) = listener.accept()?;
        to_local.send(Command::Connected)?;
        println!("New connection: {}", stream.peer_addr().unwrap());
        handle_stream(stream, &to_local, &from_local, "client")?;
    }
}

#[must_use]
pub fn connect() -> (Receiver<Command>, Sender<Command>) {
    let (to_local, from_server) = std::sync::mpsc::channel::<Command>();
    let (to_server, from_local) = std::sync::mpsc::channel::<Command>();
    thread::spawn(|| try_connect_thread(to_local, from_local));
    (from_server, to_server)
}

fn try_connect_thread(to_local: Sender<Command>, from_local: Receiver<Command>) {
    if let Err(e) = connect_thread(&to_local, &from_local) {
        println!("Server thread error: {}", e);
    }
}

fn connect_thread(
    to_local: &Sender<Command>,
    from_local: &Receiver<Command>,
) -> Result<(), AnyError> {
    let stream = TcpStream::connect(&format!("77.225.240.187:{}", PORT))?;
    handle_stream(stream, to_local, from_local, "server")
}

fn handle_stream(
    mut stream: TcpStream,
    to_local: &Sender<Command>,
    from_local: &Receiver<Command>,
    remote_name: &str,
) -> Result<(), AnyError> {
    let mut buf = [0; 500];
    // TODO: make sure the buffer is clean everytime before reading. use the de_bin(offset, buf)
    stream
        .set_read_timeout(Some(Duration::from_millis(10)))
        .unwrap();
    while remote_to_local(remote_name, to_local, &mut stream, &mut buf)?
        && local_to_remote(remote_name, from_local, &mut stream)?
    {}
    Ok(())
}

fn remote_to_local(
    remote_name: &str,
    to_local: &Sender<Command>,
    stream: &mut TcpStream,
    buf: &mut [u8; 500],
) -> Result<bool, AnyError> {
    let should_continue = match stream.read(buf) {
        Ok(size) => {
            if size == 0 {
                println!("nothing more to read");
                false
            } else {
                let command = Command::deserialize_bin(buf)?;
                println!(
                    "read {} bytes from {} for command {:?}",
                    size, remote_name, command
                );
                to_local.send(command)?;
                true
            }
        }
        Err(e) => {
            match e.kind() {
                ErrorKind::WouldBlock | ErrorKind::TimedOut => {
                    // println!("would have blocked");
                    true
                }
                _ => {
                    println!(
                        "An error occurred, terminating connection with {}. Error: {}",
                        stream.peer_addr().unwrap(),
                        e
                    );
                    stream.shutdown(std::net::Shutdown::Both).unwrap();
                    false
                }
            }
        }
    };
    Ok(should_continue)
}

fn local_to_remote(
    remote_name: &str,
    from_local: &Receiver<Command>,
    stream: &mut TcpStream,
) -> Result<bool, AnyError> {
    while let Ok(answer) = from_local.try_recv() {
        println!("sending command to {}: {:?}", remote_name, answer);
        let answer_buf = answer.serialize_bin();
        stream.write(&answer_buf)?;
        // TODO: there might be some things to do if the write fails
    }
    Ok(true)
}
