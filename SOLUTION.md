# Solution
# Bug 1 [server not accepting more than one echo message]
- the self.listener.set_nonblocking(true)?;
# Design flow 1 [handling more than on client]
- using the thread crate to run each client handler in a separate thread to allow more than one client to connect to the server 
    -! problem faced during implementation
        the thread spawn function disrupts the main thread of the server affecting the server thread causing the server to only accept one message from each client causing the test to panic when check for response.is_ok(),
->  suggested fix using joinhandle.
# Design flow 2 [not filtering client messages]
- The server handle could only handle echo messages without the add request that is part of our porto buffer implementation 
-	!problem faced 
    the returned value to the client is not encoded properly result in the response value to not be Ok()
 -> to solve the problem we need to find the right way to decode the message from the client
# Design flow 3 [async implementation]
-	Consider redefining the server architecture to run async function with the use of libraries like tokio or mio which will help with client handling, resource management (CPU usage), and latency
# Design flow 4 [thread pool]
-	Implementing thread pool to limit the thread creation to prevent resource exhaustion
# Deduging improvements 
-	Put in consideration the ability to use the env_logger through the implementation of the _init_ function and adding to the toml file env_logger = "0.10"

# Tests that could be added
- client disconnecting without activity to test if the server will auto shutdown
- server stress with many clients 
