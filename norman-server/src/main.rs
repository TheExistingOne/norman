use std::net::{TcpListener, TcpStream};
use std::io::prelude::*;
use std::{env, process};

use norman_server::{ThreadPool, UserOptions};

fn main() {
    let user_args = UserOptions::new(env::args()).unwrap_or_else(|err| {
        eprintln!("Problem parsing arguments: {}", err);
        process::exit(1);
    });

    let listener = TcpListener::bind("127.0.0.1:1315").unwrap();
    let pool = ThreadPool::new(user_args.thread_count);

    for stream in listener.incoming() {
        let stream = stream.unwrap();

        pool.execute(|| {
            handle_request(stream);
        });
    }

    fn handle_request(mut stream: TcpStream) {
        let mut buffer = [0; 512];
        stream.read(&mut buffer).unwrap();

        let sh_req = b"SHELL NORMAN/0.1";

        let (status_line, return_data) = if buffer.starts_with(sh_req) {
            ("NORMAN/0.1 200 OK\r\n\r\n", "Request Ok")
        } else {
            ("NORMAN/0.1 500 FAIL\r\n\r\n", "Request Failed. Not supported norman call")
        };

        let contents = return_data;

        let response = format!("{}{}", status_line, contents);

        stream.write(response.as_bytes()).unwrap();
        stream.flush().unwrap();
    }
}
