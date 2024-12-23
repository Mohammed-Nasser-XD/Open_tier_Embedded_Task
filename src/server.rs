use crate::message::{client_message, ClientMessage, EchoMessage,AddRequest, AddResponse};
use log::{error, info, warn};
use prost::Message;
use std::{
    io::{self, ErrorKind, Read, Write},
    net::{TcpListener, TcpStream},
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    thread,
    time::Duration,
};

struct Client {
    stream: TcpStream,
}

impl Client {
    pub fn new(stream: TcpStream) -> Self {
        Client { stream }
    }

    pub fn handle(&mut self) -> io::Result<()> {
        let mut buffer = [0; 512];
        // Read data from the client
        let bytes_read = self.stream.read(&mut buffer)?;
        if bytes_read == 0 {
            info!("Client disconnected.");
            return Ok(());
        }

        //Check the message type between EchoMessage and AddRequest

        let message_type =  ClientMessage::decode(&buffer[..bytes_read])?;

        // fix the decoding issue
        // if message_type == client_message::Message::EchoMessage() {
        //     let echo_message = EchoMessage::decode(&buffer[..bytes_read])
        //         .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "Failed to decode EchoMessage"))?;
        // }else {
        //     let addrequest = AddRequest::decode(&buffer[..bytes_read])
        //         .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "Failed to decode AddRequest"))?;
        // }

        match message_type.message {
            Some(client_message::Message::EchoMessage(echo_message)) => {
                // let echo_message = EchoMessage::decode(&buffer[..bytes_read])
                //     .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "Failed to decode EchoMessage"))?;
                info!("Received EchoMessage: {}", echo_message.content);
                println!("Received EchoMessage: {}", echo_message.content);
    
                // Echo back the message
                let payload = echo_message.content.encode_to_vec();
                self.stream.write_all(&payload)?;
                self.stream.flush()?;
            }
            Some(client_message::Message::AddRequest(addrequest)) => {
                // let addrequest = AddRequest::decode(&buffer[..bytes_read])
                //     .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "Failed to decode AddRequest"))?;
                info!("Received AddRequest: {} + {}", addrequest.a, addrequest.b);
                println!("Received AddRequest: {} + {}", addrequest.a, addrequest.b);
    
                // Calculate the sum and send AddResponse
                let result = addrequest.a + addrequest.b;
                println!("Result: {}", result);
                let response = AddResponse { result };
                let payload = result.encode_to_vec();
                self.stream.write_all(&payload)?;
                self.stream.flush()?;
            }
            _ => {
                error!("Unknown message type");
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "Unknown message type",
                ));
            }
        }
        Ok(())
    }
}

pub struct Server {
    listener: TcpListener,
    is_running: Arc<AtomicBool>,
}

impl Server {
    /// Creates a new server instance
    pub fn new(addr: &str) -> io::Result<Self> {
        let listener = TcpListener::bind(addr)?;
        let is_running = Arc::new(AtomicBool::new(false));
        Ok(Server {
            listener,
            is_running,
        })
    }

    /// Runs the server, listening for incoming connections and handling them
    pub fn run(&self) -> io::Result<()> {
        self.is_running.store(true, Ordering::SeqCst); // Set the server as running
        info!("Server is running on {}", self.listener.local_addr()?);
        let running_flag = self.is_running.clone();

        // turn off non-blocking mode to be able to accept connections
        self.listener.set_nonblocking(false)?;
        // Set up a Ctrl + C handler to stop the server gracefully
        ctrlc::set_handler(move||{
            running_flag.store(false, Ordering::SeqCst);
        }).expect("Error setting Ctrl +C handler");
 
            
        while self.is_running.load(Ordering::SeqCst) {
            match self.listener.accept() {
                Ok((_stream, addr)) => {
                    info!("New client connected: {}", addr);
                    
                    // Handle the client request
                    for stream in self.listener.incoming() {
                        match stream {
                            Ok(stream) => {
                                let is_running = self.is_running.clone();
                                // Spawn a new thread to handle clients concurrently
                                thread::spawn(move || {
                                    let mut client = Client::new(stream);
                                    while is_running.load(Ordering::SeqCst) {
                                        match client.handle() {
                                            Ok(_) => {
                                                info!("Message handled successfully.");
                                                // Continue waiting for the next message
                                            }
                                            Err(ref e) if e.kind() == io::ErrorKind::ConnectionAborted => {
                                                info!("Client disconnected.");
                                                break; // Exit loop if the client disconnects
                                            }
                                            Err(e) => {
                                                error!("Error handling client: {}", e);
                                                break; // Exit loop on other errors
                                            }
                                            // JoinHandle::new() to fix the threading issue
                                        }
                                    }
                                    info!("Client disconnected.");
                                });
                            }
                            Err(ref e) if e.kind() == ErrorKind::WouldBlock => {
                                // No incoming connections, sleep briefly to reduce CPU usage
                                thread::sleep(Duration::from_millis(100));
                            }
                            Err(e) => {
                                error!("Error accepting connection: {}", e);
                            }
                        }
                    }
                }
                Err(ref e) if e.kind() == ErrorKind::WouldBlock => {
                    // No incoming connections, sleep briefly to reduce CPU usage
                    thread::sleep(Duration::from_millis(100));
                }
                Err(e) => {
                    error!("Error accepting connection: {}", e);
                }
            }
        }

        info!("Server stopped.");
        Ok(())
    }

    /// Stops the server by setting the `is_running` flag to `false`
    pub fn stop(&self) {
        if self.is_running.load(Ordering::SeqCst) {
            self.is_running.store(false, Ordering::SeqCst);
            info!("Shutdown signal sent.");
        } else {
            warn!("Server was already stopped or not running.");
        }
    }
}
