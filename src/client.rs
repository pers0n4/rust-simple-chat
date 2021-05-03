use std::io::{ErrorKind, Read, Write};
use std::net::TcpStream;
use std::sync::mpsc;
use std::thread;

const BUFFER_SIZE: usize = 32;

pub fn main(addr: &str) {
    let mut stream = TcpStream::connect(addr).unwrap();
    stream.set_nonblocking(true).unwrap();
    println!("Server connected: {}", addr);

    let (tx, rx) = mpsc::channel::<String>();

    // 통신을 위한 스레드 생성
    thread::spawn(move || loop {
        let mut buffer = vec![0; BUFFER_SIZE];
        // 서버로부터 데이터를 받은 경우 출력
        match stream.read_exact(&mut buffer) {
            Ok(_) => {
                let message = buffer
                    .into_iter()
                    .take_while(|&x| x != 0)
                    .collect::<Vec<_>>();
                let message = String::from_utf8(message).unwrap();

                println!("{}", message);
            }
            Err(ref error) if error.kind() == ErrorKind::WouldBlock => (),
            Err(_) => {
                println!("Server disconnected");
                break;
            }
        }

        // 클라이언트에서 메시지를 입력한 경우 서버로 전송
        match rx.try_recv() {
            Ok(message) => {
                let mut buffer = message.clone().into_bytes();
                buffer.resize(BUFFER_SIZE, 0);
                stream.write_all(&buffer).unwrap();
            }
            Err(mpsc::TryRecvError::Empty) => (),
            Err(mpsc::TryRecvError::Disconnected) => break,
        }
        thread::sleep(std::time::Duration::from_millis(100));
    });

    println!("Message:");
    // 새로운 메시지를 받고 서버로 전송
    loop {
        let mut buffer = String::new();
        std::io::stdin().read_line(&mut buffer).unwrap();
        let message = buffer.trim().to_string();
        if message == ":quit" || tx.send(message).is_err() {
            break;
        }
    }
}
