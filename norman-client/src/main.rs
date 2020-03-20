use std::net::{TcpStream, TcpListener};
use norman_client::*;
use std::io::prelude::*;

//use norman_client::UserOptions;
fn main() {
    // let user_args = UserOptions::new(env::args()).unwrap_or_else(|err| {
    //     eprintln!("Problem parsing arguments: {}", err);
    //     process::exit(1);
    // });
    let _guard = sentry::init("https://3d034496ffe8417f988b81f617ee032c@sentry.io/4616084");

    sentry::integrations::panic::register_panic_handler();


    let mut stream = match TcpStream::connect("127.0.0.1:7878") {
        Ok(tcp_stream) => tcp_stream,
        Err(error) => {
            panic!("Issue connecting to remote host: {:?}", error)
        },
    };

    let listener = match TcpListener::bind("127.0.0.1:7575") {
        Ok(listener) => listener,
        Err(error) => {
            panic!("Error binding listener: {:?}", error)
        },
    };

    let packet = NormanPacket::new(String::from("NORMAN/0.1"), true, Service::SHELL, RequestType::REQUEST, Status::FINE{code: 200}, String::from("None"), String::from("echo \"Hello World\""), false);

    stream.write(packet.as_string().as_bytes()).unwrap();

    for stream in listener.incoming().take(1) {
        let mut stream = stream.unwrap();

        let mut buffer = [0; 512];
        stream.read(&mut buffer).unwrap();

        let return_packet = NormanPacket::from_string(String::from_utf8(buffer.to_vec()).unwrap());

        println!("{}", return_packet.data.data);
    }
}