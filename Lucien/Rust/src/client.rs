use std;
use std::io::*;
use std::net::TcpStream;
use std::thread;

static leave_msg: &str = "BYE";

fn get_entry() -> String {
    let mut buf = String::new();

    stdin().read_line(&mut buf).unwrap();
    buf.replace("\n", "").replace("\r", "")
}

fn write_to_server(stream: TcpStream) {
    let mut writer = BufWriter::new(&stream);

    macro_rules! send {
        ($line:expr) => ({
            try!(writeln!(writer, "{}", $line));
            try!(writer.flush());
        })
    }

    loop {
        match &*get_entry() {
            "/quit" => {
                println!("Disconnecting...");
                send!("BYE");
                println!("Disconnected!");
                return ();
            }
            line => {
                send!(line);
            }
        }
    }
}

fn exchange_with_server(stream: TcpStream) {
    let server = stream.peer_addr().unwrap();
    println!("Connected to {}", server);
    // Buffered reading and writing
    let mut reader = BufReader::new(&stream);
    let mut writer = BufWriter::new(&stream);

    println!("Enter `/quit` when you want to leave");

    macro_rules! receive {
        () => ({
            let mut line = String::new();
            match reader.read_line(&mut line) {
                Ok(len) => {
                    if len == 0 {
                        // Reader is at EOF. Could use ErrorKind::UnexpectedEOF, but still unstable.
                        let ret = std::io::Error::new(std::io::ErrorKind::Other, "test");
                        return Err(ret): std::result::Result<&str, std::io::Error>;
                        // return Err(Error::new(ErrorKind::Other, "unexpected EOF"));
                    }
                    line.pop();
                }
                Err(e) => { return Err(e); }
            };
            line
        })
    }

    thread::spawn(move || {
        write_to_server(stream.try_clone().unwrap());
    });

    match (|| loop {
        let input = receive!();
        println!("{}", input);
    })()
    {
        Ok(_) => {
            println!("Left?");
        }
        Err(e) => {
            println!("Disappeared? {}", e);
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
