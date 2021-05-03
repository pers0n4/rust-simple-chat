use std::io::{ErrorKind, Read, Write};
use std::net::TcpListener;
use std::sync::mpsc;
use std::thread;

const BUFFER_SIZE: usize = 32;

pub fn main(addr: &str) {
    let listener = TcpListener::bind(addr).unwrap();
    listener.set_nonblocking(true).unwrap();
    println!("Server listening: {}", addr);

    // 송수신을 위한 통신 채널
    let (tx, rx) = mpsc::channel::<String>();
    // 서버에 접속 중인 클라이언트 목록
    let mut clients = Vec::new();

    loop {
        if let Ok((mut socket, addr)) = listener.accept() {
            // 서버로 송신할 수 있는 채널을 클라이언트로 전달
            let tx = tx.clone();
            clients.push(socket.try_clone().unwrap());
            println!("Client connected: {}", addr);

            // move 클로저를 통한 소유권 전달 (프로세스 -> 스레드)
            thread::spawn(move || loop {
                let mut buffer = vec![0; BUFFER_SIZE];

                // 클라이언트가 데이터를 전송한 경우
                match socket.read_exact(&mut buffer) {
                    Ok(_) => {
                        let message = buffer
                            .into_iter()
                            .take_while(|&x| x != 0)
                            .collect::<Vec<_>>();
                        let message = String::from_utf8(message).unwrap();
                        let message = format!("{}: {:?}", addr, message);

                        println!("{}", message);
                        tx.send(message).unwrap();
                    }
                    Err(ref error) if error.kind() == ErrorKind::WouldBlock => (),
                    Err(_) => {
                        println!("Client disconnected: {}", addr);
                        break;
                    }
                }
                sleep();
            });
        }

        // 서버에서 데이터를 수신한 경우
        if let Ok(message) = rx.try_recv() {
            // 접속 중인 모든 클라이언트에 데이터 전달
            clients = clients
                .into_iter()
                .filter_map(|mut client| {
                    let mut buffer = message.clone().into_bytes();
                    buffer.resize(BUFFER_SIZE, 0);

                    client.write_all(&buffer).map(|_| client).ok()
                })
                .collect::<Vec<_>>();
        }
        sleep();
    }
}

fn sleep() {
    thread::sleep(std::time::Duration::from_millis(100));
}
