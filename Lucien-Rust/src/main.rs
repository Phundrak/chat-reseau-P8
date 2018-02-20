use std::env;
use std::net::{TcpListener, TcpStream};
use std::thread;
#[allow(unused_imports)]
use std::io::{stdin, stdout, Read, Write};
#[allow(unused_imports)]
use std::sync::{Arc, Mutex};

///////////////////////////////////////////////////////////////////////////////
//                                                                           //
//                                   Client                                  //
//                                                                           //
///////////////////////////////////////////////////////////////////////////////

fn get_entry() -> String {
    let mut buf = String::new();

    stdin().read_line(&mut buf).unwrap();
    buf.replace("\n", "").replace("\r", "")
}

fn exchange_with_server(mut stream: TcpStream) {
    let stdout = std::io::stdout();
    let mut io = stdout.lock();
    let buf = &mut [0; 3];

    println!("Enter `quit` when you want to leave");
    loop {
        write!(io, "> ").unwrap();
        io.flush().unwrap();
        match &*get_entry() {
            "quit" => {
                println!("bye!");
                return;
            }
            "exit" => {
                println!("bye!");
                return;
            }
            line => {
                write!(stream, "{}\n", line).unwrap();
                match stream.read(buf) {
                    Ok(received) => {
                        if received < 1 {
                            println!("Perte de la connexion avec le serveur");
                            return;
                        }
                    }
                    Err(_) => {
                        println!("Perte de la connexion avec le serveur");
                        return;
                    }
                }
                // println!("Réponse du serveur : {:?}", buf);
                let reponse = String::from_utf8(buf.to_vec()).unwrap();
                println!("Réponse du serveur : {}", reponse);
            }
        }
    }
}

fn client(server_address: String) {
    println!("Tentative de connexion a serveur...");
    match TcpStream::connect(server_address) {
        Ok(stream) => {
            println!("Connexion au serveur réussie !");
            exchange_with_server(stream);
        }
        Err(e) => {
            println!("La connection au serveur a échoué : {}", e);
        }
    }
}

///////////////////////////////////////////////////////////////////////////////
//                                                                           //
//                                   Server                                  //
//                                                                           //
///////////////////////////////////////////////////////////////////////////////

fn handle_client(mut stream: &TcpStream, adresse: &str) {
    let mut msg: Vec<u8> = Vec::new();
    loop {
        let buf = &mut [0; 10];

        match stream.read(buf) {
            Ok(received) => {
                // si on a reçu 0 octet, ça veut dire que le client s'est déconnecté
                if received < 1 {
                    println!("Client disconnected {}", adresse);
                    return;
                }
                let mut x = 0;

                for c in buf {
                    // si on a dépassé le nombre d'octets reçus, inutile de continuer
                    if x >= received {
                        break;
                    }
                    x += 1;
                    if *c == '\n' as u8 {
                        println!(
                            "message reçu {} : {}",
                            adresse,
                            // on convertit maintenant notre buffer en String
                            String::from_utf8(msg).unwrap()
                        );

                        stream.write(b"ok\n").unwrap();

                        msg = Vec::new();
                    } else {
                        msg.push(*c);
                    }
                }
            }
            Err(_) => {
                println!("Client disconnected {}", adresse);
                return;
            }
        }
    }
}

fn serveur(port: String) {
    println!("Port: {}", port);
    let mut serv = String::from("127.0.0.1:");
    serv.push_str(&port);
    let listener = TcpListener::bind(serv.to_string()).unwrap();

    println!("En attente d’un client...");

    // Multi-client ///////////////////////////////////////////////////////////
    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                let adresse = match stream.peer_addr() {
                    Ok(addr) => format!("[adresse : {}]", addr),
                    Err(_) => "inconnue".to_owned(),
                };

                println!("Nouveau client {}", adresse);
                thread::spawn(move || handle_client(&stream, &*adresse));
            }
            Err(e) => {
                println!("La connexion du client a échoué : {}", e);
            }
        }
        println!("En attente d’un autre client...");
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() == 2 {
        ///////////////////////////////////////////////////////////////////////
        //                           Server opened                           //
        ///////////////////////////////////////////////////////////////////////
        println!("Opening server on port {}", args[1]);
        serveur(args[1].clone());
    } else if args.len() == 3 {
        ///////////////////////////////////////////////////////////////////////
        //                           Client opened                           //
        ///////////////////////////////////////////////////////////////////////
        println!("Client connecting on server {}:{}", args[1], args[2]);
        let mut serv = if args[1] == String::from("localhost") {
            String::from("127.0.0.1")
        } else {
            args[1].clone()
        };
        serv.push(':');
        serv.push_str(&args[2]);
        client(serv);
    } else {
        println!("Usage: {} [server ip] port", args[0]);
    }
}
