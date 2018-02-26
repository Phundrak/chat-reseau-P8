use std::net::{TcpListener, TcpStream};
use std::io::{Read, Write};
use std::thread;

fn handle_client(mut stream: &TcpStream, adresse: &str, name: String) {
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
                            "message reçu {}({}) : {}",
                            name,
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

pub fn serveur(port: String) {
    println!("Port: {}", port);
    let mut serv = String::from("127.0.0.1:");
    serv.push_str(&port);
    let listener = TcpListener::bind(serv.to_string()).unwrap();

    println!("En attente d’un client...");

    // Multi-client ///////////////////////////////////////////////////////////
    for stream in listener.incoming() {
        match  stream {
            Ok(stream) => {
                let adresse = match stream.peer_addr() {
                    Ok(addr) => format!("[adresse : {}]", addr),
                    Err(_) => "inconnue".to_owned(),
                };

                let name = String::from("Toto");

                println!("Nouveau client {}", adresse);
                thread::spawn(move || handle_client(&stream, &*adresse, name));
            }
            Err(e) => {
                println!("La connexion du client a échoué : {}", e);
            }
        }
        println!("En attente d’un autre client...");
    }
}
