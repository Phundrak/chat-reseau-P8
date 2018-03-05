extern crate bufstream;
use std::io::{BufRead, Write};
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::str::FromStr;
use std::sync::{mpsc, Arc, RwLock};
use std::sync::mpsc::{Receiver, Sender};
use std::thread::spawn;
use self::bufstream::BufStream;

fn handle_connection(
    stream: &mut BufStream<TcpStream>,
    chan: Sender<String>,
    arc: Arc<RwLock<Vec<String>>>,
) {
    stream.write(b"Welcome this server!\n").unwrap();
    stream
        .write(b"Please input your username (max. 20chars): ")
        .unwrap();
    stream.flush().unwrap();

    let mut name = String::new();
    stream.read_line(&mut name).unwrap();
    let name = name.trim_right();
    stream
        .write_fmt(format_args!("Hello, {}!\n", name))
        .unwrap();
    stream.flush().unwrap();

    let mut pos = 0;
    loop {
        {
            let lines = arc.read().unwrap();
            for i in pos..lines.len() {
                stream.write_fmt(format_args!("{}", lines[i])).unwrap();
                pos = lines.len();
            }
        }
        stream.write(b" > ").unwrap();
        stream.flush().unwrap();

        let mut reads = String::new();
        stream.read_line(&mut reads).unwrap();
        if reads.trim().len() != 0 {
            chan.send(format!("[{}] said: {}", name, reads)).unwrap();
        }
    }
}

pub fn serveur(addr: String) {
    // Ouverture de la connexion sur socket
    let addr = SocketAddr::from_str(&addr).unwrap();
    // Ajout d’un listener Tcp sur le socket
    let listener = TcpListener::bind(addr).unwrap();

    // création des receveurs et envoyeurs de strings asynchrones
    let (sender, receiver): (Sender<String>, Receiver<String>) = mpsc::channel();
    let arc: Arc<RwLock<Vec<String>>> = Arc::new(RwLock::new(Vec::new()));
    let arc_w = arc.clone();

    // boucle infinie en parallèle pour recevoir des messages
    spawn(move || {
        loop {
            // lit le message depuis le receveur
            let msg = receiver.recv().unwrap();
            print!("DEBUG: message {}", msg);
            {
                let mut arc_w = arc_w.write().unwrap();
                arc_w.push(msg);
            }
        }
    });

    // Réception des clients
    for stream in listener.incoming() {
        match stream {
            Err(e) => println!("Erreur écoute : {}", e),
            Ok(mut stream) => {
                println!(
                    "Nouvelle connexion de {} vers {}",
                    stream.peer_addr().unwrap(),
                    stream.local_addr().unwrap()
                );
                let sender = sender.clone();
                let arc = arc.clone();
                spawn(move || {
                    let mut stream = BufStream::new(stream);
                    handle_connection(&mut stream, sender, arc);
                });
            }
        }
    }
}
