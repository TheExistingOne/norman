//Packet Structure

#[derive(PartialEq, Debug)]
pub enum Service {
    SHELL,
    DOCKER,
    AWS,
    UNKNOWN,
}

#[derive(PartialEq, Debug)]
pub enum RequestType{
    REQUEST,
    RETURN,
    TEST,
    ERROR,
}

#[derive(PartialEq, Debug)]
pub enum Status {
    FINE{code: i32},
    ERROR{code: i32},
    TEST{code:i32},
    MALFORMED{code:i32},
}

#[derive(PartialEq, Debug)]
pub struct Header {
    pub version: String, //Format NORMAN/<ver>
    pub return_output: bool,
    pub service: Service,
}

#[derive(PartialEq, Debug)]
pub struct Metadata {
    pub req_type: RequestType,
    pub status: Status,
    pub uid: i32,
}

#[derive(PartialEq, Debug)]
pub struct Encryption {
    pub encoding_type: String,
    pub key: String,
}

#[derive(PartialEq, Debug)]
pub struct Data {
    pub data: String,
}

#[derive(PartialEq, Debug)]
pub struct Terminator {
    pub multi_packet: bool,
    pub term_string: String,
}

#[derive(PartialEq, Debug)]
pub struct NormanPacket {
    pub header: Header,
    pub meta: Metadata,
    pub encryption: Encryption,
    pub data: Data,
    pub terminator: Terminator, //Should be NORMAN/END
}

impl NormanPacket {
    pub fn new (ver: String, ret_out: bool, service: Service, req_type: RequestType, status: Status, enc_type: String, data: String, multi_packet: bool) -> NormanPacket {
        NormanPacket {
            header: Header {
                version: String::from("NORMAN/") + &ver,
                return_output: ret_out,
                service: service,
            },
            meta: Metadata {
                req_type: req_type,
                status: status,
                uid: 0,
            },
            encryption: Encryption {
                encoding_type: enc_type,
                key: String::from(" "),
            },
            data: Data {
                data,
            },
            terminator: Terminator {
                multi_packet: multi_packet,
                term_string: String::from("NORMAN/END"),
            },
        }
    }

    pub fn as_string(packet: NormanPacket) -> String {
        let mut packet_string: String = String::new();

        //Concatenate Header
        packet_string = packet_string + 
            //Version
            &packet.header.version + "|" +
            //Return Flag
            match &packet.header.return_output{
                true => "true",
                false => "false",
            } + "|" +
            //Target Service
            match &packet.header.service {
                Service::AWS => "AWS",
                Service::DOCKER => "DOCKER",
                Service::SHELL => "SHELL",
                Service::UNKNOWN => "UNKNOWN"
            } + "|";
        
        //Concatenate Metadata
        packet_string = packet_string + 
            //Type of packet
            match &packet.meta.req_type {
                RequestType::REQUEST => "REQUEST",
                RequestType::RETURN => "RETURN",
                RequestType::TEST => "TEST",
                RequestType::ERROR => "ERROR",
            } + "|" +
            //Status of packet
            match &packet.meta.status {
                Status::FINE{code: 200} => "200 OK",
                Status::ERROR{code: 500} => "500 ERR",
                Status::TEST{code: 100} => "100 TEST",
                _ => "505 MALFORMED"
            } + "|" +
            //Packet ID
            &packet.meta.uid.to_string()  + "|";
        
        //Concatenate Data
        packet_string = packet_string +
            //Packet Data
            &packet.data.data + "|";

        //Concatenate Terminator
        packet_string = packet_string +
            //Concatenated packet flag
            match &packet.terminator.multi_packet{
                true => "true",
                false => "false",
            } + "|" +
            //Packet terminator
            &packet.terminator.term_string;

        packet_string
    }

    pub fn from_string (packet_string: String) -> NormanPacket {
        let mut packet_components = packet_string.split("|");

        let mut version: String;
        let mut return_output: bool;
        let mut service: Service;
        
        let mut req_type: RequestType;
        let mut status: Status;
        
        let mut encoding_type: String;
        
        let mut data: String;
        
        let mut multi_packet: bool;
        
        if packet_components.clone().count() == 11 {
            version = packet_components.next().unwrap().to_string();
            return_output = match packet_components.next().unwrap() {
                "true" => true,
                "false" => false,
                _ => true
            };
            service = match packet_components.next().unwrap() {
                "SHELL" => Service::SHELL,
                "AWS" => Service::AWS,
                "DOCKER" => Service::DOCKER,
                _ => Service::UNKNOWN,
            };

            req_type = match packet_components.next().unwrap() {
                "REQUEST" => RequestType::REQUEST,
                "RETURN" => RequestType::RETURN,
                "TEST" => RequestType::TEST,
                _ => RequestType::ERROR,
            };
            status = match packet_components.next().unwrap() {
                "200 OK" => Status::FINE{code: 200},
                "500 ERR" => Status::ERROR{code: 500},
                "100 TEST" => Status::TEST{code: 100},
                _ => Status::MALFORMED{code: 505},
            };

            encoding_type = packet_components.next().unwrap().to_string();

            data = packet_components.next().unwrap().to_string();

            multi_packet = match packet_components.next().unwrap() {
                "true" => true,
                "false" => false,
                _ => false,
            };
        } else {
            panic!("Recieved malformed packet!");
        }

        NormanPacket::new(version, return_output, service, req_type, status, encoding_type, data, multi_packet)
    }
}


pub struct Target {
    pub ip: String,
    pub port: String,
}

//Parse User Input
pub struct UserOptions {
    pub target: Target,
}

impl UserOptions {
    pub fn new(mut args: std::env::Args) -> Result<UserOptions, &'static str> {
        args.next();
        let ip = match args.next() {
            Some(arg) => arg,
            None => return Err("No ip provided. \n Syntax: norman <ip> <port>"),
        };
        let port = match args.next() {
            Some(arg) => arg,
            None => return Err("No port provided \n Syntax: norman <ip> <port>")
        };

        let target = Target{ip, port};

        Ok(UserOptions{target})
    }
}

#[test]
fn valid_packets() {
    let shell_packet = NormanPacket::new("0.1".to_string(), true, Service::SHELL, RequestType::REQUEST, Status::FINE{code:200}, "None".to_string(), "echo \"Hello from norman\"".to_string(), false);
    let shell_expected_string = String::from("NORMAN/0.1|true|SHELL|REQUEST|200 OK|0|echo \"Hello from norman\"|false|NORMAN/END");
    let aws_packet = NormanPacket::new("0.1".to_string(), false, Service::AWS, RequestType::REQUEST, Status::FINE{code:200}, "None".to_string(), "exec start \"Ubuntu 19.10 Server\"".to_string(), false);
    let aws_expected_string = String::from("NORMAN/0.1|false|AWS|REQUEST|200 OK|0|exec start \"Ubuntu 19.10 Server\"|false|NORMAN/END");

    assert_eq!(shell_expected_string, NormanPacket::as_string(shell_packet));
    assert_eq!(aws_expected_string, NormanPacket::as_string(aws_packet));
}

#[test]
fn error_packets() {
    let malformed_packet = NormanPacket::new("0.1".to_string(), true, Service::SHELL, RequestType::REQUEST, Status::FINE{code:699}, "None".to_string(), "echo \"Hello from norman\"".to_string(), false);
    let malformed_expected_string = String::from("NORMAN/0.1|true|SHELL|REQUEST|505 MALFORMED|0|echo \"Hello from norman\"|false|NORMAN/END");
    let err_packet = NormanPacket::new("0.1".to_string(), true, Service::SHELL, RequestType::RETURN, Status::ERROR{code:500}, "None".to_string(), "echo \"Hello from norman\"".to_string(), false);
    let err_expected_string = String::from("NORMAN/0.1|true|SHELL|RETURN|500 ERR|0|echo \"Hello from norman\"|false|NORMAN/END");

    assert_eq!(malformed_expected_string, NormanPacket::as_string(malformed_packet));
    assert_eq!(err_expected_string, NormanPacket::as_string(err_packet));
}

#[test]
fn valid_string_conversion() {
    let shell_packet = NormanPacket::new("0.1".to_string(), true, Service::SHELL, RequestType::REQUEST, Status::FINE{code:200}, "None".to_string(), "echo \"Hello from norman\"".to_string(), false);
    let shell_expected_string = String::from("NORMAN/0.1|true|SHELL|REQUEST|200 OK|0|echo \"Hello from norman\"|false|NORMAN/END");
    let aws_packet = NormanPacket::new("0.1".to_string(), false, Service::AWS, RequestType::REQUEST, Status::FINE{code:200}, "None".to_string(), "exec start \"Ubuntu 19.10 Server\"".to_string(), false);
    let aws_expected_string = String::from("NORMAN/0.1|false|AWS|REQUEST|200 OK|0|exec start \"Ubuntu 19.10 Server\"|false|NORMAN/END");

    assert_eq!(NormanPacket::from_string(shell_expected_string), shell_packet);
    assert_eq!(NormanPacket::from_string(aws_expected_string), aws_packet);
}

#[test]
fn error_string_conversion() {
    let malformed_packet = NormanPacket::new("0.1".to_string(), true, Service::SHELL, RequestType::REQUEST, Status::FINE{code:699}, "None".to_string(), "echo \"Hello from norman\"".to_string(), false);
    let malformed_expected_string = String::from("NORMAN/0.1|true|SHELL|REQUEST|505 MALFORMED|0|echo \"Hello from norman\"|false|NORMAN/END");
    let err_packet = NormanPacket::new("0.1".to_string(), true, Service::SHELL, RequestType::RETURN, Status::ERROR{code:500}, "None".to_string(), "echo \"Hello from norman\"".to_string(), false);
    let err_expected_string = String::from("NORMAN/0.1|true|SHELL|RETURN|500 ERR|0|echo \"Hello from norman\"|false|NORMAN/END");

    assert_eq!(NormanPacket::from_string(malformed_expected_string), malformed_packet);
    assert_eq!(NormanPacket::from_string(err_expected_string), err_packet);
}