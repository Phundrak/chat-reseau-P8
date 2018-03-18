use std::io::*;
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::thread;
use std::sync::{Arc, Mutex, MutexGuard};
use std::collections::HashMap;

// Map for all connected clients containing their name and stream
type UserMapValue = (String, TcpStream);
type UserMap = HashMap<SocketAddr, UserMapValue>;

fn distribute_message(msg: &str, not_to: &SocketAddr, lock: &mut MutexGuard<UserMap>) {
    for (other_client, entry) in (*lock).iter() {
        if other_client != not_to {
            let other_name = &entry.0;
            let other_stream = &entry.1;
            match (|| -> Result<()> {
                let mut writer = BufWriter::new(other_stream);
                try!(writeln!(writer, "{}", msg));
                try!(writer.flush());
                return Ok(());
            })()
            {
                Ok(_) => {}
                Err(e) => {
                    println!(
                        "Client {} <{}> disappeared during message distribution: {}",
                        other_client, other_name, e
                    );
                }
            }
        }
    }
}

fn disconnect_user(name: &str, client: &SocketAddr, lock: &mut MutexGuard<UserMap>) {
    (*lock).remove(&client);
    distribute_message(&format!("{} left", name), client, lock);
}

fn handle_client(stream: TcpStream, clients: Arc<Mutex<UserMap>>) {
    // Get client IP and port
    let client = stream.peer_addr().unwrap();
    println!("New connection from {}", client);

    // Buffered reading and writing
    let mut reader = BufReader::new(&stream);
    let mut writer = BufWriter::new(&stream);

    // Write an entire line to the client
    // Can fail on IO errors, du to try! macro
    macro_rules! send {
        ($line:expr) => ({
            try!(writeln!(writer, "{}", $line));
            try!(writer.flush());
        })
    }

    // Read an entire line from the client
    // Can fail on IO errors or when EOF is reached
    macro_rules! receive {
        () => ({
            let mut line = String::new();
            match reader.read_line(&mut line) {
                Ok(len) => {
                    if len == 0 {
                        // Reader is at EOF.
                        return Err(Error::new(ErrorKind::Other, "unexpected EOF"));
                    }
                    line.pop();
                }
                Err(e) => {
                    return Err(e);
                }
            };
            line
        })
    }

    // Initialization: Ask user for his name
    let name = match (|| {
        send!("Welcome!");
        send!("Please enter your name:");
        let name = receive!();
        println!("Client {} identified as {}", client, name);
        send!("DEBUG: You can now type messages. Leave this chat with the request `BYE`.");
        Ok(name)
    })()
    {
        Ok(name) => name,
        Err(e) => {
            println!("Client {} disappeared during initialization: {}", client, e);
            return ();
        }
    };

    // Add user to global map. Lock will be released at the end of the scope
    {
        let mut lock = clients.lock().unwrap();
        (*lock).insert(client, (name.clone(), stream.try_clone().unwrap()));
        distribute_message(&format!("{} joined", name), &client, &mut lock);
    }

    // Chat loop: Receive messages from users
    match (|| {
        loop {
            let input = receive!();
            if input == "BYE" {
                send!("Bye!");
                return Ok(());
            }

            // Distribute message
            println!("{} <{}>: {}", client, name, input);
            {
                let mut lock = clients.lock().unwrap();
                distribute_message(&format!("<{}>: {}", name, input), &client, &mut lock);
            }
        }
    })()
    {
        Ok(_) => {
            println!("Client {} <{}> left", client, name);
        }
        Err(e) => {
            println!(
                "Client {} <{}> disappeared during chat: {}",
                client, name, e
            );
        }
    }

    // Remove user from global map
    {
        let mut lock = clients.lock().unwrap();
        disconnect_user(&name, &client, &mut lock);
    }
}

pub fn serveur(addr: String) {
    // Manage UserMap in a mutex
    let clients = Arc::new(Mutex::new(HashMap::new()));
    let serv_addr = addr.clone();

    // Start a TCP Listener
    let listener = match TcpListener::bind(serv_addr.as_str()) {
        Ok(listener) => listener,
        Err(e) => panic!("Could not read start TCP listener: {}", e),
    };

    println!("Successfully started the server on {}", serv_addr);

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                let clients = clients.clone();
                thread::spawn(move || {
                    //connection succeeded
                    handle_client(stream, clients)
                });
            }
            Err(e) => {
                writeln!(stderr(), "Connection failed: {}", e).unwrap();
            }
        }
    }

    // close the socket server
    drop(listener);
}
