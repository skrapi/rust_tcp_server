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

    for stream in listener.incoming() {
        let stream = stream.unwrap();

        pool.execute(|| {
            handle_connection(stream);
        });
    }

    println!("Shutting down.");
}

fn handle_connection(mut stream: TcpStream) {
    // Get mac address from geyser controller
    // TODO add read of mac address
    let mut buffer = [0 as u8; 17];
    let len = stream.peek(&mut buffer).unwrap();
    println!("Length: {}", len);
    stream.read(&mut buffer).unwrap();
    let filename = format!("{}{}", &(String::from_utf8_lossy(&buffer))[..], ".bl2");
    println!("Filename: {}", filename);

    // Read in firmware image
    let image: Vec<u8> = read_in_image(&filename);

    // Check if image exists
    if image.len() == 1 {
        println!("Image was not found");
        return;
    } else {
        println!("Image found, starting bootload process");
    }

    // Send Header and wait for response
    // 0x13ff returned will mean it is erasing
    stream.set_nonblocking(true).unwrap();

    let mut position: usize = 0;

    loop {
        stream.write(&image[0..64]).unwrap();

        stream.flush().unwrap();
        println!("Header Sent");

        thread::sleep(Duration::from_secs(1));

        let mut buffer = [0 as u8; 32];
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
        println!("Data: {:?}", buffer);

        let erasing: u8 = 19;
        if buffer.contains(&erasing) {
            println!("Erasing");
            break;
        }
    }
    // Start of sending packets
    position += 64;

    stream.set_nonblocking(false).unwrap();

    loop {
        let mut buffer = [0 as u8; 1];

        let _ = match stream.peek(&mut buffer) {
            Ok(_) => stream.read_exact(&mut buffer).unwrap(),
            Err(_) => {
                println!("No data");
                continue;
            }
        };

        let ready: u8 = 17;
        if buffer.contains(&ready) {
            println!("Ready for image");

            stream.read(&mut buffer).unwrap();
            break;
        }
    }

    loop {
        let mut buffer = [0 as u8; 1];
        let _ = match stream.peek(&mut buffer) {
            Ok(_) => {
                stream.read(&mut buffer).unwrap();
                println!("Data: {:?}", buffer);
            }
            Err(_) => {
                println!("No data");
                continue;
            }
        };

        let mut length_of_data = buffer[0] as usize;
        if length_of_data == 0 {
            continue;
        }
        if position >= image.len() {
            break;
        } else if position + length_of_data > image.len() {
            length_of_data = image.len() - position;
        }
        stream
            .write(&image[position..(position + length_of_data)])
            .unwrap();

        stream.flush().unwrap();
        position += length_of_data;
        println!(
            "Progress {} %",
            100.0 * position as f64 / image.len() as f64
        );
    }
}

fn read_in_image(filename: &str) -> Vec<u8> {
    let image = match fs::read(filename) {
        Ok(file) => file,
        Err(_) => vec![0],
    };

    image
}
