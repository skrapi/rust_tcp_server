use std::fs;
use std::io::prelude::*;
use std::net::TcpListener;
use std::net::TcpStream;

use std::thread;
use std::time::Duration;

use hello::ThreadPool;

fn main() {
    let listener = TcpListener::bind("127.0.0.1:7878").unwrap();
    let pool = ThreadPool::new(4);

    for stream in listener.incoming().take(2) {
        let stream = stream.unwrap();

        pool.execute(|| {
            handle_connection(stream);
        });
    }

    println!("Shutting down.");
}

fn handle_connection(mut stream: TcpStream) {
    let image = read_in_image();

    println!("Image read in");
    loop {
        stream.write(&image[0..64]).unwrap();

        stream.flush().unwrap();

        thread::sleep(Duration::from_secs(1));

        let mut buffer = [0; 10];
        let len = match stream.peek(&mut buffer) {
            Ok(num) => num,
            Err(_) => continue,
        };

        stream.read(&mut buffer).unwrap();
        println!("Received {} bytes", len);

        if buffer.starts_with(b"0x13ff") {
            break;
        }
    }
}

fn read_in_image() -> Vec<u8> {
    let image = fs::read("firmware_geyser_controller.production.bl2").unwrap();
    image
}
