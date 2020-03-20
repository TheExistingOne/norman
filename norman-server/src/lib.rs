use std::thread;
use std::sync::{mpsc, Mutex, Arc};

//Parse User Input
pub struct UserOptions {
    pub thread_count: usize,
}

impl UserOptions {
    pub fn new(mut args: std::env::Args) -> Result<UserOptions, &'static str> {
        args.next();
        let thread_count = match args.next() {
            Some(arg) => arg,
            None => return Err("No thread count provided. \n This is the number of threads the application should create"),
        };
        
        let thread_count = thread_count.trim().parse().expect("Thread count must be a number");
        
        Ok(UserOptions{thread_count})
    }
}

// ThreadPool System
enum Message {
    NewJob(Job),
    Terminate,
}

pub struct ThreadPool {
    workers: Vec<Worker>,
    sender: mpsc::Sender<Message>,
}

type Job = Box<dyn FnOnce() + Send + 'static>;

impl ThreadPool {
    /// Create a new ThreadPool.
    /// 
    /// The size is the number of threads in the pool
    /// 
    /// # Panics
    /// 
    /// The `new` function will panic if the size is zero
    pub fn new (size: usize) -> ThreadPool {
        assert!(size > 0);

        let (sender, receiver) = mpsc::channel();

        let receiver = Arc::new(Mutex::new(receiver));

        let mut workers = Vec::with_capacity(size);

        for id in 0..size {
            workers.push(Worker::new(id, Arc::clone(&receiver)));
        }

        ThreadPool {
            workers,
            sender,
        }
    }

    /// Run a function on the next available thread in the pool.
    /// 
    /// f is the function you want to run.
    pub fn execute<F>(&self, f: F)
        where
            F: FnOnce() + Send + 'static
    {
        let job = Box::new(f);

        self.sender.send(Message::NewJob(job)).unwrap();
    }
}

impl Drop for ThreadPool {
    fn drop(&mut self) {
        println!("Sending terminate message to all workers.");

        for _ in &mut self.workers {
            self.sender.send(Message::Terminate).unwrap();
        } 

        println!("Shutting down all workers.");

        for worker in &mut self.workers {
            println!("Shuuting down worker {}", worker.id);

            if let Some(thread) = worker.thread.take() {
                thread.join().unwrap();
            }
        }
    }
}

pub struct Worker {
    id: usize,
    thread: Option<thread::JoinHandle<()>>,
}

impl Worker {
    fn new(id: usize, receiver: Arc<Mutex<mpsc::Receiver<Message>>>) ->
        Worker {

        let thread = thread::spawn(move ||{
            loop {
                let message = receiver.lock().unwrap().recv().unwrap();

                match message {
                    Message::NewJob(job) => {
                        println!("Worker {} got a job; executing.", id);

                        job();
                    },
                    Message::Terminate => {
                        println!("Worker {} was told to terminate.", id);

                        break;
                    },
                }
            }
        });

        Worker {
            id,
            thread: Some(thread),
        }
    }
}

//Packet Structure

#[derive(PartialEq, Clone, Debug)]
pub enum Service {
    SHELL,
    DOCKER,
    AWS,
    UNKNOWN,
}

#[derive(PartialEq, Clone, Debug)]
pub enum RequestType{
    REQUEST,
    RETURN,
    TEST,
    ERROR,
}

#[derive(PartialEq, Clone, Debug)]
pub enum Status {
    FINE{code: i32},
    ERROR{code: i32},
    TEST{code:i32},
    MALFORMED{code:i32},
}

#[derive(PartialEq, Clone, Debug)]
pub struct Header {
    pub version: String, //Format NORMAN/<ver>
    pub return_output: bool,
    pub service: Service,
}

#[derive(PartialEq, Clone, Debug)]
pub struct Metadata {
    pub req_type: RequestType,
    pub status: Status,
    pub uid: i32,
}

#[derive(PartialEq, Clone, Debug)]
pub struct Encryption {
    pub encoding_type: String,
    pub key: String,
}

#[derive(PartialEq, Clone, Debug)]
pub struct Data {
    pub data: String,
}

#[derive(PartialEq, Clone, Debug)]
pub struct Terminator {
    pub multi_packet: bool,
    pub term_string: String,
}

#[derive(PartialEq, Clone, Debug)]
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
                version: ver,
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

    pub fn as_string(&self) -> String {
        let mut packet_string: String = String::new();

        //Concatenate Header
        packet_string = packet_string + 
            //Version
            &self.header.version + "|" +
            //Return Flag
            match &self.header.return_output{
                true => "true",
                false => "false",
            } + "|" +
            //Target Service
            match &self.header.service {
                Service::AWS => "AWS",
                Service::DOCKER => "DOCKER",
                Service::SHELL => "SHELL",
                Service::UNKNOWN => "UNKNOWN"
            } + "|";
        
        //Concatenate Metadata
        packet_string = packet_string + 
            //Type of packet
            match &self.meta.req_type {
                RequestType::REQUEST => "REQUEST",
                RequestType::RETURN => "RETURN",
                RequestType::TEST => "TEST",
                RequestType::ERROR => "ERROR",
            } + "|" +
            //Status of packet
            match &self.meta.status {
                Status::FINE{code: 200} => "200 OK",
                Status::ERROR{code: 500} => "500 ERR",
                Status::TEST{code: 100} => "100 TEST",
                _ => "505 MALFORMED"
            } + "|" +
            //Packet ID
            &self.meta.uid.to_string()  + "|";
        //Concatenate Encryption
            packet_string = packet_string +
            //Encryption Type
            &self.encryption.encoding_type + "|" +
            &self.encryption.key + "|";
        //Concatenate Data
        packet_string = packet_string +
            //Packet Data
            &self.data.data + "|";

        //Concatenate Terminator
        packet_string = packet_string +
            //Concatenated packet flag
            match &self.terminator.multi_packet{
                true => "true",
                false => "false",
            } + "|" +
            //Packet terminator
            &self.terminator.term_string;

        packet_string
    }

    pub fn from_string (packet_string: String) -> NormanPacket {
        let mut packet_components = packet_string.split("|");

        let version: String;
        let return_output: bool;
        let service: Service;
        
        let req_type: RequestType;
        let status: Status;
        
        let encoding_type: String;
        
        let data: String;
        
        let multi_packet: bool;
        
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

            packet_components.next();

            encoding_type = packet_components.next().unwrap().to_string();

            packet_components.next();

            data = packet_components.next().unwrap().to_string();

            multi_packet = match packet_components.next().unwrap() {
                "true" => true,
                "false" => false,
                _ => false,
            };
        } else {
            let component_count = packet_components.clone().count();
            let mut panic_string = String::new();
            for component in packet_components {
                panic_string = format!("{}{}{}", panic_string, component, String::from(","));
            }
            panic!("Malformed packet! Expeceted 9 components but found {}. Packet read {}", component_count, panic_string);
        }

        NormanPacket::new(version, return_output, service, req_type, status, encoding_type, data, multi_packet)
    }
}