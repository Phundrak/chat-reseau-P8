extern crate bufstream;
use std::net::TcpStream;
use std::io::{stdin, stdout, Read, Write};
// use std::sync::mpsc;
// use std::sync::mpsc::{Receiver, Sender};
use std::thread::spawn;
// use self::bufstream::BufStream;

fn get_entry() -> String {
    let mut buf = String::new();

    stdin().read_line(&mut buf).unwrap();
    buf.replace("\n", "").replace("\r", "")
}

fn read_from_server(
    mut stream: TcpStream,
) {
    let buff = &mut [0; 1024];
    let stdout = stdout();
    let mut io = stdout.lock();
    loop {
        match stream.read(buff) {
            Ok(received) => {
                if received < 1 {
                    // println!("Perte de connexion avec le serveur");
                    write!(io, "Perte de connexion avec le serveur\n").unwrap();
                    io.flush().unwrap();
                    return;
                }
            }
            Err(_) => {
                // println!("Perte de connexion avec le serveur");
                write!(io, "Perte de connexion avec le serveur\n").unwrap();
                io.flush().unwrap();
                return;
            }
        }
        let reponse = String::from_utf8(buff.to_vec()).unwrap();
        write!(io, "{}", reponse).unwrap();
        io.flush().unwrap();
        // println!("From server: {}", reponse);
    }
}

fn exchange_with_server(
    mut stream: TcpStream
) {
    let stdout = stdout();
    let mut io = stdout.lock();
    let _buff = &mut [0; 1024];

    let stream_cpy = stream.try_clone().unwrap();
    spawn(move || {
        // let stream_cpy = stream.try_clone().unwrap();
        read_from_server(stream_cpy);
    });

    println!("Enter `quit` or `exit` when you want to leave");

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
                // match stream.read(buff) {
                //     Ok(received) => {
                //         if received < 1 {
                //             println!("Perte de la connexion avec le serveur");
                //             return;
                //         }
                //     }
                //     Err(_) => {
                //         println!("Perte de la connexion avec le serveur");
                //         return;
                //     }
                // }
                // println!("Réponse du serveur : {}", buf);
                // let reponse = String::from_utf8(buf.to_vec()).unwrap();
                // println!("Réponse du serveur : {}", reponse);
            }
        }
    }
}

// fn exchange_with_server(stream: TcpStream) {
//     let (chan, recv): (Sender<String>, Receiver<String>) = mpsc::channel();
//     // let buf = &mut [0; 1024];
//     spawn(move || {
//         loop {
//             let msg = recv.recv().unwrap();
//             println!("{}", msg);
//         }
//     });
//     println!("Enter `quit` or `exit` when you want to leave");
//     loop {
//         match &*get_entry() {
//             "quit" => {
//                 println!("bye!");
//                 return;
//             }
//             "exit" => {
//                 println!("bye!");
//                 return;
//             }
//             line => {
//                 chan.send(format!("{}", line)).unwrap();
//             }
//         }
//     }
// }

pub fn client(server_address: String) {
    println!("Tentative de connexion a serveur...");
    match TcpStream::connect(server_address) {
        Ok(stream) => {
            println!("Connexion au serveur réussie !");
            exchange_with_server(stream);
        }
        Err(e) => {
            println!("La connection au serveur a échoué : {}", e);
            return;
        }
    }
}
