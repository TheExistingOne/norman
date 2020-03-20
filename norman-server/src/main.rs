use std::net::{TcpListener, TcpStream, Shutdown};
use std::io::prelude::*;
use std::{env, process};
use cmd_lib::*;

use norman_server::*;

fn main() {
    let user_args = UserOptions::new(env::args()).unwrap_or_else(|err| {
        eprintln!("Problem parsing arguments: {}", err);
        process::exit(1);
    });

    let listener = TcpListener::bind("127.0.0.1:7878").unwrap();
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

        let mut ret_stream = TcpStream::connect("127.0.0.1:7575").unwrap();

        let packet = String::from_utf8(buffer.to_vec()).unwrap();

        println!("Got norman packet: {}", packet);

        let packet = NormanPacket::from_string(packet);

        let comm_out = run_fun!("{}", &packet.data.data);

        let response = NormanPacket::new(packet.header.version, packet.header.return_output, packet.header.service, RequestType::RETURN, Status::FINE{code: 200}, String::from("None"), comm_out.unwrap(), false);        

        ret_stream.write(response.as_string().as_bytes()).unwrap();
        stream.flush().unwrap();
        ret_stream.flush().unwrap();
        ret_stream.shutdown(Shutdown::Both).unwrap();
    }
}
