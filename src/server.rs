use std::io::{ErrorKind, Read, Write};
use std::net::TcpListener;
use std::sync::mpsc;
use std::thread;

const BUFFER_SIZE: usize = 32;

pub fn main(addr: &str) {
    let listener = TcpListener::bind(addr).unwrap();
    listener.set_nonblocking(true).unwrap();
    println!("Server listening: {}", addr);

    let (tx, rx) = mpsc::channel::<String>();
    let mut clients = Vec::new();

    loop {
        if let Ok((mut socket, addr)) = listener.accept() {
            let tx = tx.clone();
            clients.push(socket.try_clone().unwrap());
            println!("Client connected: {}", addr);

            // move 클로저를 통한 소유권 전달 (프로세스 -> 스레드)
            thread::spawn(move || loop {
                let mut buffer = vec![0; BUFFER_SIZE];

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

        if let Ok(message) = rx.try_recv() {
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
