use macroquad::prelude::*;
use std::io::{Read, Write};
use std::net::{Ipv4Addr, TcpListener, TcpStream};
use std::thread;

fn handle_client(mut stream: TcpStream) {
    println!("spawned thread");
    let mut data = [0 as u8; 50]; // using 50 byte buffer
    while match stream.read(&mut data) {
        Ok(size) => {
            if size == 0 {
                println!("nothing more to read");
                false
            } else {
                let something = data[0];
                let answer = [size as u8, something];
                stream.write(&answer).unwrap();
                true
            }
        }
        Err(_) => {
            println!(
                "An error occurred, terminating connection with {}",
                stream.peer_addr().unwrap()
            );
            stream.shutdown(std::net::Shutdown::Both).unwrap();
            false
        }
    } {}
}

fn main() {
    let listener = TcpListener::bind("0.0.0.0:31415").unwrap();
    println!("Server listening on port 31415");
    loop {
        match listener.accept() {
            Ok((stream, _socket_addr)) => {
                println!("New connection: {}", stream.peer_addr().unwrap());
                thread::spawn(|| handle_client(stream));
            }
            Err(e) => {
                println!("Error: {}", e);
            }
        }
    }
}
