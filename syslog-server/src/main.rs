use std::thread;
use std::net::{UdpSocket, TcpListener, TcpStream};
use std::io::{Read};
use rsyslog::Message;

fn print_message(row: &str) -> Result<(), String>{
    //println!("{}", row);
    let message: Message = rsyslog::Message::parse(row).map_err(|e| e.to_string())?;
    println!("{:?}", message);

    Ok(())
}

fn handle_udp_client(mut socket: UdpSocket) {
    let mut buf = [0; 4096];
    loop {
        let sock = socket.try_clone();
        match socket.recv_from(&mut buf) {
            Ok((amt, src)) => {
                thread::spawn(move || {
                    //println!("Handling connection from {}", &src);
                    let buf = &mut buf[..amt];
                    let row = std::str::from_utf8(&buf).unwrap();
                    print_message(row);
                });
            },
            Err(err) => {
                eprintln!("Err: {}", err);
            }
        }
    }
}

fn handle_tcp_client(mut stream: TcpStream) {
    let mut data = String::new(); // using 50 byte buffer
    stream.read_to_string(&mut data).unwrap();
    print_message(&data);
}

fn main() -> std::io::Result<()> {
    thread::spawn(move|| {
        let tcp_listener = TcpListener::bind("0.0.0.0:1468").unwrap();
        // accept connections and process them, spawning a new thread for each one
        println!("TCPServer listening on port 1468");
        for stream in tcp_listener.incoming() {
            match stream {
                Ok(stream) => {
                    //println!("New connection: {}", stream.peer_addr().unwrap());
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
        // close the socket server
        drop(tcp_listener);
    });


    let socket = UdpSocket::bind("0.0.0.0:514")?;
    println!("UDP listening on port 514");
    handle_udp_client(socket);
    Ok(())
}