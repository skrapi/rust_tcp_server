use std::fs;
use std::io::prelude::*;
use std::net::TcpListener;
use std::net::TcpStream;

use std::thread;
use std::time::Duration;

use hello::ThreadPool;

fn main() {
    let ip_address = fs::read_to_string("ip_address.txt").unwrap();

    let listener = TcpListener::bind(ip_address).unwrap();
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

    loop {
        stream.write(&image[0..64]).unwrap();

        stream.flush().unwrap();

        let mut buffer = [0; 10];
        stream.read(&mut buffer).unwrap();

        println!("Received data: {:?}", buffer);

        if buffer.starts_with(b"0x13ff") {
            break;
        }
    }
}

fn read_in_image() -> Vec<u8> {
    let image = fs::read("firmware_geyser_controller.production.bl2").unwrap();
    image
}
