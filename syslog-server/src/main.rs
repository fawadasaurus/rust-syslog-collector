use std::thread;
use std::net::{UdpSocket, TcpListener, TcpStream, Shutdown};
use std::io::{Read, Write};

fn handle_udp_client(mut socket: UdpSocket) {
    let mut buf = [0; 4096];
    loop {
        let sock = socket.try_clone();
        match socket.recv_from(&mut buf) {
            Ok((amt, src)) => {
                thread::spawn(move || {
                    //println!("Handling connection from {}", &src);
                    let buf = &mut buf[..amt];
                    println!("{}", std::str::from_utf8(&buf).unwrap());
                });
            },
            Err(err) => {
                eprintln!("Err: {}", err);
            }
        }
    }
}

fn handle_tcp_client(mut stream: TcpStream) {
    let mut data = [0 as u8; 50]; // using 50 byte buffer
    while match stream.read(&mut data) {
        Ok(size) => {
            // echo everything!
            println!("{}", std::str::from_utf8(&data[0..size]).unwrap());
            true
        },
        Err(_) => {
            println!("An error occurred, terminating connection with {}", stream.peer_addr().unwrap());
            stream.shutdown(Shutdown::Both).unwrap();
            false
        }
    } {}
}

fn main() -> std::io::Result<()> {
    thread::spawn(move|| {
        let tcp_listener = TcpListener::bind("0.0.0.0:1468").unwrap();
        // accept connections and process them, spawning a new thread for each one
        println!("TCPServer listening on port 1468");
        for stream in tcp_listener.incoming() {
            match stream {
                Ok(stream) => {
                    println!("New connection: {}", stream.peer_addr().unwrap());
                    thread::spawn(move|| {
                        // connection succeeded
                        handle_tcp_client(stream)
                    });
                }
                Err(e) => {
                    println!("Error: {}", e);
                    /* connection failed */
                }
            }
        }
    });


    let socket = UdpSocket::bind("127.0.0.1:8514")?;
    println!("UDP listening on port 8514");
    handle_udp_client(socket);
    Ok(())
}