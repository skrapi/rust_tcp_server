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

    stream.set_nonblocking(true).unwrap();

    let mut position = 0;
    loop {
        stream.write(&image[0..64]).unwrap();

        stream.flush().unwrap();
        println!("Header Sent");

        thread::sleep(Duration::from_secs(1));

        let mut buffer = [0; 10];
        let len = match stream.peek(&mut buffer) {
            Ok(num) => num,
            Err(_) => {
                println!("No data");
                continue;
            }
        };

        println!("About to read");
        stream.read(&mut buffer).unwrap();
        println!("Received {} bytes", len);
        println!("Data: {}", String::from_utf8_lossy(&buffer[..]));

        if buffer.starts_with(b"hello") {
            println!("Erasing");
            break;
        }
    }
    // Start of sending packets
    position += 64;

    while position <= image.len() {
        let mut buffer = [0; 10];
    }
}

fn read_in_image() -> Vec<u8> {
    let image = fs::read("firmware_geyser_controller.production.bl2").unwrap();
    image
}
