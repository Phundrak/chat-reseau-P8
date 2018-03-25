extern crate chrono;
use std::io::*;
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::thread;
use std::sync::{Arc, Mutex, MutexGuard};
use std::collections::HashMap;
use self::chrono::Local;

// TODO: add server-side controls: display clients list, kick client, shutdown

///////////////////////////////////////////////////////////////////////////////
//                     Evolution implementation protocole                    //
///////////////////////////////////////////////////////////////////////////////

/*

0.1   [X]
1.1   [X]
1.2   [X]
1.3   [X]
1.4   [X]
1.5   [X]
1.6   [X]
1.7   [X]
1.8   [X]
1.9   [X]
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

pub fn get_time() -> String {
    let date = Local::now();
    date.format("[%H:%M:%S]").to_string()
}

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
                    "{} Client {} <{}> disappeared during message distribution: {}",
                    get_time(),
                    other_client,
                    other_name,
                    e
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
    println!("{} New connection from {}", get_time(), client);

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

    // Get user's name
    let name: String = match (|| loop {
        match receive!() {
            input => {
                println!(
                    "{time} Client {addr} : {message}",
                    time = get_time(),
                    addr = client,
                    message = input
                );
                let spliced_input: Vec<&str> = input.split_whitespace().collect();
                if spliced_input.len() != 4 && spliced_input.len() != 5
                    || spliced_input[0] != "PROT"
                {
                    return Err(Error::new(ErrorKind::Other, "BAD REQ"));
                }
                if spliced_input[1] != ::PROTOCOL {
                    return Err(Error::new(ErrorKind::Other, "BAD PROT"));
                }
                if spliced_input.len() == 5 {
                    if spliced_input[2] == "CONNECT" && spliced_input[3] == "USER" {
                        let username = String::from(spliced_input[4]);
                        let mut ascii_nick = true;
                        for c in username.chars() {
                            if !c.is_ascii() {
                                ascii_nick = false;
                                println!(
                                    "{time} to client {addr} : {message}",
                                    time = get_time(),
                                    addr = client,
                                    message = "NAME FAILURE"
                                );
                                send!("NAME FAILURE");
                                break;
                            }
                        }
                        if ascii_nick {
                            let mut used = false;
                            {
                                let lock = clients.lock().unwrap();
                                for (_, entry) in (*lock).iter() {
                                    if username == entry.0 {
                                        used = true;
                                        break;
                                    }
                                }
                            }
                            if used == false {
                                println!(
                                    "{time} to client {addr} : {message}",
                                    time = get_time(),
                                    addr = client,
                                    message = "NAME OK"
                                );
                                send!("NAME OK");
                                return Ok(username);
                            } else {
                                println!(
                                    "{time} to client {addr} : {message}",
                                    time = get_time(),
                                    addr = client,
                                    message = "NAME FAILURE"
                                );
                                send!("NAME FAILURE");
                            }
                        }
                    } else {
                        return Err(Error::new(ErrorKind::Other, "BAD REQ"));
                    }
                }

                loop {
                    println!(
                        "{time} to client {addr} : {message}",
                        time = get_time(),
                        addr = client,
                        message = "NAME REQ"
                    );
                    send!("NAME REQ");
                    match receive!() {
                        input => {
                            println!(
                                "{time} Client {addr} : {message}",
                                time = get_time(),
                                addr = client,
                                message = input
                            );
                            let spliced_input: Vec<&str> = input.split_whitespace().collect();
                            if spliced_input.len() != 2 || spliced_input[0] != "NAME" {
                                return Err(Error::new(ErrorKind::Other, "BAD REQ"));
                            }
                            let username = String::from(spliced_input[1]);
                            let mut ascii_nick = true;
                            for c in username.chars() {
                                if !c.is_ascii() {
                                    ascii_nick = false;
                                    println!(
                                        "{time} to client {addr} : {message}",
                                        time = get_time(),
                                        addr = client,
                                        message = "NAME FAILURE"
                                    );
                                    send!("NAME FAILURE");
                                    break;
                                }
                            }
                            if ascii_nick {
                                let mut used = false;
                                {
                                    let lock = clients.lock().unwrap();
                                    for (_, entry) in (*lock).iter() {
                                        if username == entry.0 {
                                            used = true;
                                            break;
                                        }
                                    }
                                }
                                if used == false {
                                    println!(
                                        "{time} to client {addr} : {message}",
                                        time = get_time(),
                                        addr = client,
                                        message = "NAME OK"
                                    );
                                    send!("NAME OK");
                                    return Ok(username);
                                } else {
                                    println!(
                                        "{time} to client {addr} : {message}",
                                        time = get_time(),
                                        addr = client,
                                        message = "NAME FAILURE"
                                    );
                                    send!("NAME FAILURE");
                                }
                            }
                        }
                    }
                }
            }
        }
    })()
    {
        Ok(name) => name,
        Err(e) => {
            println!(
                "{time} client {addr} encountered an error: {err}",
                time = get_time(),
                addr = client,
                err = e
            );
            writeln!(writer, "{}", e).unwrap();
            writer.flush().unwrap();
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

    // Chat loop: Receive messages from users once connected
    match (|| loop {
        match receive!().as_str() {
            input => {
                println!(
                    "{time} {nick}@{addr}: {message}",
                    time = get_time(),
                    addr = client,
                    nick = name,
                    message = input
                );

                match input {
                    "BYE" => {
                        println!(
                            "{time} to client {addr} : {message}",
                            time = get_time(),
                            addr = client,
                            message = "BYE"
                        );
                        send!("BYE");
                        return Ok(());
                    }

                    "PING" => {
                        println!(
                            "{time} to client {addr} : {message}",
                            time = get_time(),
                            addr = client,
                            message = "NAME FAILURE"
                        );
                        send!("PONG");
                    }

                    "REQ CLIENTS" => {
                        let mut lock = clients.lock().unwrap();
                        send_clients_name(&client, &mut lock);
                    }
                    input => {
                        let spliced_input: Vec<&str> = input.split_whitespace().collect();
                        match spliced_input[0] {
                            "MSG" => {
                                let mut message = String::new();
                                for i in 1..spliced_input.len() {
                                    message.push_str(spliced_input[i]);
                                }
                                {
                                    let mut lock = clients.lock().unwrap();
                                    distribute_message(
                                        &format!("{}", input),
                                        &client,
                                        &mut lock,
                                        true,
                                    );
                                }
                            }
                            _ => {
                                println!(
                                    "{time} to client {addr} : \"{message}\", cause : {inmessage}",
                                    time = get_time(),
                                    addr = client,
                                    message = "BAD REQ",
                                    inmessage = input
                                );
                                send!("BAD REQ");
                            }
                        }
                    }
                }
                // {
                //     let mut lock = clients.lock().unwrap();
                //     distribute_message(&format!("{}", input), &client, &mut lock, true);
                // }
            }
        }
    })()
    {
        Ok(_) => {
            println!("{} Client {} <{}> left", get_time(), client, name);
        }
        Err(e) => {
            println!(
                "{} Client {} <{}> disappeared during chat: {}",
                get_time(),
                client,
                name,
                e
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

    println!(
        "{} Successfully started the server on {}",
        get_time(),
        serv_addr
    );

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
