use std::net::TcpStream;
use std::io::{Read, Write};
use std::io::{stdin, stdout};

fn get_entry() -> String {
    let mut buf = String::new();

    stdin().read_line(&mut buf).unwrap();
    buf.replace("\n", "").replace("\r", "")
}

fn exchange_with_server(mut stream: TcpStream) {
    let stdout = stdout();
    let mut io = stdout.lock();
    let buf = &mut [0; 1024];

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
                // println!("Réponse du serveur : {}", buf);
                let reponse = String::from_utf8(buf.to_vec()).unwrap();
                println!("Réponse du serveur : {}", reponse);
            }
        }
    }
}

pub fn client(server_address: String) {
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
