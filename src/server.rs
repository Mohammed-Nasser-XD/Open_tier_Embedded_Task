use crate::message::{EchoMessage, AddRequest, AddResponse};
use log::{error, info, warn};
use prost::Message;
use ctrlc;
use std::{
    io::{self, ErrorKind, Read, Write}, 
    net::{TcpListener, TcpStream}, 
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    }, 
    thread, 
    time::Duration
};
pub struct Server {
    listener: TcpListener,
    is_running: Arc<AtomicBool>,
}

impl Server {
    /// Creates a new server instance
    pub fn new(addr: &str) -> io::Result<Self> {
        let listener: TcpListener = TcpListener::bind(addr)?;
        let is_running: Arc<AtomicBool> = Arc::new(AtomicBool::new(false));
        Ok(Server {
            listener,
            is_running,
        })
    }

    /// Runs the server, listening for incoming connections and handling them
    pub fn run(&self) -> io::Result<()> {
        self.is_running.store(true, Ordering::SeqCst);  // Set the server as running
        let running_flag = self.is_running.clone();

        info!("Server is running on {}", self.listener.local_addr()?);

        // Set the listener to non-blocking mode
        self.listener.set_nonblocking(true)?;

        //

        ctrlc::set_handler(move||{
            running_flag.store(false, Ordering::SeqCst);
        }).expect("Error setting Ctrlc handler");
 
            
        while self.is_running.load(Ordering::SeqCst) {
            match self.listener.accept() {
                Ok((stream, addr)) => {
                    info!("New client connected: {}", addr);
                    let is_running = self.is_running.clone();

                    // Handle the client request
                    thread::spawn(move || {
                        let mut client = Client::new(stream);
                        while is_running.load(Ordering::SeqCst) {
                            if let Err(e) = client.handle() {
                                error!("Error handling client: {}", e);
                                break;
                            }
                        }
                        info!("Client disconnected.");
                    });

                    // let mut client = Client::new(stream);
                    // while self.is_running.load(Ordering::SeqCst) {   
                    //     if let Err(e) = client.handle() {
                    //         error!("Error handling client: {}", e);
                    //         break;
                    //     }
                    // }
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

struct Client {
    stream: TcpStream,
}

impl Client {
    pub fn new(stream: TcpStream) -> Self {
        Client { stream }
    }
    
    pub fn handle(&mut self) -> io::Result<()> {
        let mut buffer = vec![0; 1024];
        // Read data from the client
        let bytes_read = self.stream.read(&mut buffer)?;
        if bytes_read == 0 {
            info!("Client disconnected.");
            return Err(io::Error::new(
                io::ErrorKind::ConnectionAborted, 
                "Client disconnected")) ;
        }

        if let Ok(message) = EchoMessage::decode(&buffer[..bytes_read]) {
            info!("Received: {}", message.content);
            println!("Received: {}", message.content);
            // Echo back the message
            let payload = message.encode_to_vec();
            self.stream.write_all(&payload)?;
            self.stream.flush()?;
        }else if let Ok(add_request) = AddRequest::decode(&buffer[..bytes_read]) {
            info!("Received: {} + {}", add_request.a, add_request.b);
            println!("Received: {} + {}", add_request.a, add_request.b);
            // Calculate the sum
            let result = add_request.a + add_request.b;
            let response = AddResponse { result };
            let payload = response.encode_to_vec();
            self.stream.write_all(&payload)?;
            self.stream.flush()?;
        } else {
            error!("Failed to decode message");
            return Err(io::Error::new(
                io::ErrorKind::InvalidData, 
                "Failed to decode message"));
        }
        Ok(())
    }
}

