extern crate syslog;

use std::collections::HashMap;
use std::env;
use std::thread;
use std::time;
use syslog::{Facility, Formatter5424};

fn main() {
    let mut server_host = env::var("SERVER_HOST").unwrap_or("".to_string());

    println!("server host is {}", server_host);
    if server_host == "" {
        println!("server host not found, defaulting to 0.0.0.0");
        server_host = "0.0.0.0".to_string();
    }

    let udp_server = format!("{}:8514", server_host);
    let tcp_server = format!("{}:8468", server_host);

    let formatter_udp = Formatter5424 {
        facility: Facility::LOG_USER,
        hostname: None,
        process: "syslog-client-udp".into(),
        pid: 0,
    };

    let formatter_tcp = Formatter5424 {
        facility: Facility::LOG_USER,
        hostname: None,
        process: "syslog-client-tcp".into(),
        pid: 0,
    };

    let udp_thread = thread::spawn(move || {
        thread::sleep(time::Duration::from_secs(5));
        match syslog::udp(formatter_udp, "0.0.0.0:8123", &udp_server) {
            Err(e) => println!("impossible to connect to syslog: {:?}", e),
            Ok(mut writer) => {
                writer
                    .err((1, HashMap::new(), "hello from UDP"))
                    .expect("could not write error message");
            }
        }
    });

    let tcp_thread = thread::spawn(move || {
        thread::sleep(time::Duration::from_secs(5));
        for n in 1..10 {
            thread::sleep(time::Duration::from_secs(1));
            match syslog::tcp(formatter_tcp.clone(), &tcp_server) {
                Err(e) => println!("impossible to connect to syslog: {:?}", e),
                Ok(mut writer) => {
                    let log = format!("{}{}", "Hello from TCP ", n);
                    writer
                        .err((1, HashMap::new(), log))
                        .expect("could not write error message");
                }
            }
        }
    });

    udp_thread.join();
    tcp_thread.join();
}
