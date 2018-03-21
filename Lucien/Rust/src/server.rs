use std::io::*;
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::thread;
use std::sync::{Arc, Mutex, MutexGuard};
use std::collections::HashMap;

///////////////////////////////////////////////////////////////////////////////
//                     Evolution implementation protocole                    //
///////////////////////////////////////////////////////////////////////////////

/*

1.1   [ ]
1.2   [ ]
1.3   [ ]
1.4   [ ]
1.5   [ ]
1.6   [ ]
1.7   [X]
1.8   [X]
1.9   [ ]
2.1   [X]
2.2   [X]
3.1   [ ] // pas utile avec Rust
3.2   [X]
4.1.1 [X]
4.1.2 [X]
4.2.1 [X]
4.2.2 [-]

*/

///////////////////////////////////////////////////////////////////////////////
//                                   TYPES                                   //
///////////////////////////////////////////////////////////////////////////////

// Map for all connected clients containing their name and stream
type UserMapValue = (String, TcpStream);
type UserMap = HashMap<SocketAddr, UserMapValue>;

///////////////////////////////////////////////////////////////////////////////
//                                    CODE                                   //
///////////////////////////////////////////////////////////////////////////////

fn distribute_message(
    msg: &str,
    not_to: &SocketAddr,
    lock: &mut MutexGuard<UserMap>,
    everyone: bool,
) {
    let mut name = String::new();
    for (client, entry) in (*lock).iter() {
        if client == not_to {
            name = entry.0.clone();
            break;
        }
    }
    for (other_client, entry) in (*lock).iter() {
        let other_name = &entry.0;
        let other_stream = &entry.1;
        if everyone == false && other_client == not_to {
            continue;
        }
        match (|| -> Result<()> {
            let mut writer = BufWriter::new(other_stream);
            // test if message begins with "MSG " /////////////////////////
            if &msg[..4] == "MSG " {
                try!(writeln!(writer, "FROM {} {}", name, msg));
            } else {
                try!(writeln!(writer, "{}", msg));
            }
            ///////////////////////////////////////////////////////////////
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

fn send_clients_name(to: &SocketAddr, lock: &mut MutexGuard<UserMap>) {
    let mut clients = String::new();
    for (client, entry) in (*lock).iter() {
        clients.push_str(&format!(
            "{}{} ",
            &entry.0.trim(),
            if client == to { "(you)" } else { "" }
        ));
    }
    let clients = clients.trim();
    for (client, entry) in (*lock).iter() {
        if client == to {
            let stream = &entry.1;
            let mut writer = BufWriter::new(stream);
            writeln!(writer, "LIST CLIENTS {}", clients).unwrap();
            writer.flush().unwrap();
            return;
        }
    }
}

fn disconnect_user(name: &str, client: &SocketAddr, lock: &mut MutexGuard<UserMap>) {
    (*lock).remove(&client);
    distribute_message(&format!("LOGOUT {}", name), client, lock, true);
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
        distribute_message(&format!("JOIN {}", name), &client, &mut lock, false);
    }

    writeln!(writer, "WELCOME").unwrap();
    writer.flush().unwrap();

    // Chat loop: Receive messages from users
    match (|| loop {
        match receive!().as_str() {
            "BYE" => {
                send!("BYE");
                return Ok(());
            }
            "PING" => {
                send!("PONG");
            }
            "REQ CLIENTS" => {
                let mut lock = clients.lock().unwrap();
                send_clients_name(&client, &mut lock);
            }
            input => {
                println!("{} <{}>: {}", client, name, input);
                {
                    let mut lock = clients.lock().unwrap();
                    distribute_message(&format!("{}", input), &client, &mut lock, true);
                }
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
