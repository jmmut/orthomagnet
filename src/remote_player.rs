use crate::AnyError;
use nanoserde::{DeBin, SerBin};
use std::io::{ErrorKind, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::mpsc::{Receiver, Sender};
use std::thread;
use std::time::Duration;

#[derive(SerBin, DeBin, Debug)]
pub enum Command {
    StoneHover { x: i32, y: i32 },
    StopStoneHover,
    Connected,
}

#[must_use]
pub fn serve() -> (Receiver<Command>, Sender<Command>) {
    let (from_client_sender, from_client_receiver) = std::sync::mpsc::channel::<Command>();
    let (to_client_sender, to_client_receiver) = std::sync::mpsc::channel::<Command>();
    thread::spawn(|| try_server_thread(from_client_sender, to_client_receiver));
    (from_client_receiver, to_client_sender)
}

fn try_server_thread(from_client: Sender<Command>, to_client: Receiver<Command>) {
    if let Err(e) = server_thread(from_client, to_client) {
        println!("Server thread error: {}", e);
    }
}
fn server_thread(
    mut from_client: Sender<Command>,
    mut to_client: Receiver<Command>,
) -> Result<(), AnyError> {
    let listener = TcpListener::bind("127.0.0.1:31415").unwrap();
    println!("Server listening on port 31415");
    loop {
        let (stream, _socket_addr) = listener.accept()?;
        from_client.send(Command::Connected)?;
        println!("New connection: {}", stream.peer_addr().unwrap());
        handle_client(stream, &mut from_client, &mut to_client)?;
    }
}

fn handle_client(
    mut stream: TcpStream,
    to_server: &mut Sender<Command>,
    to_client: &mut Receiver<Command>,
) -> Result<(), AnyError> {
    let mut data = [0 as u8; 500];
    // TODO: make sure the buffer is clean everytime before reading. use the de_bin(offset, buf)
    stream
        .set_read_timeout(Some(Duration::from_millis(10)))
        .unwrap();
    loop {
        let should_continue = match stream.read(&mut data) {
            Ok(size) => {
                if size == 0 {
                    println!("nothing more to read");
                    false
                } else {
                    println!("read {} bytes from client", size);
                    let command = Command::deserialize_bin(&data)?;
                    to_server.send(command)?;
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
                            "An error occurred, terminating connection with {}",
                            stream.peer_addr().unwrap()
                        );
                        stream.shutdown(std::net::Shutdown::Both).unwrap();
                        false
                    }
                }
            }
        };
        if !should_continue {
            break;
        }
        while let Ok(answer) = to_client.try_recv() {
            println!("received command in thread from client {:?}", answer);
            let answer_buf = answer.serialize_bin();
            stream.write(&answer_buf).unwrap();
        }
    }
    Ok(())
}

#[must_use]
pub fn connect() -> (Receiver<Command>, Sender<Command>) {
    let (from_server_sender, from_server_receiver) = std::sync::mpsc::channel::<Command>();
    let (to_server_sender, to_server_receiver) = std::sync::mpsc::channel::<Command>();
    thread::spawn(|| try_connect_thread(from_server_sender, to_server_receiver));
    (from_server_receiver, to_server_sender)
}

fn try_connect_thread(from_server: Sender<Command>, to_server: Receiver<Command>) {
    if let Err(e) = connect_thread(from_server, to_server) {
        println!("Server thread error: {}", e);
    }
}
fn connect_thread(
    from_server: Sender<Command>,
    to_server: Receiver<Command>,
) -> Result<(), AnyError> {
    let mut stream = TcpStream::connect("127.0.0.1:31415")?;
    stream
        .set_read_timeout(Some(Duration::from_millis(10)))
        .unwrap();
    let mut buf = [0; 500];
    loop {
        match stream.read(&mut buf) {
            Ok(_) => {
                let command = DeBin::deserialize_bin(&buf)?;
                from_server.send(command)?;
            }
            Err(e) => {
                match e.kind() {
                    ErrorKind::WouldBlock | ErrorKind::TimedOut => {
                        // println!("would have blocked");
                    }
                    _ => Err(e)?,
                }
            }
        }

        while let Ok(answer) = to_server.try_recv() {
            println!("sending command from thread to server {:?}", answer);
            let answer_buf = answer.serialize_bin();
            stream.write(&answer_buf).unwrap();
        }
    }
}
