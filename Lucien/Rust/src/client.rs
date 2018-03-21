use std;
use std::io::*;
use std::net::TcpStream;
use std::thread;

// static leave_msg: &str = "BYE";

fn get_entry() -> String {
    let mut buf = String::new();

    stdin().read_line(&mut buf).unwrap();
    buf.replace("\n", "").replace("\r", "")
}

fn write_to_server(stream: TcpStream) {
    let mut writer = BufWriter::new(&stream);

    // entrée du nom d'utilisateur
    loop {
        match &*get_entry() {
            "" => {
                continue;
            }
            "/quit" => {
                println!("Disconnecting...");
                writeln!(writer, "BYE").unwrap();
                writer.flush().unwrap();
                println!("Disconnected!");
                return ();
            }
            line => {
                let line_str : String = String::from(line);
                // let spliced: Vec<&str> = line_str.split(" ").collect();
                let spliced: Vec<&str> = line_str.split_whitespace().collect();
                if spliced.len() > 1 {
                    println!("Cannot use whitespace in username.");
                    continue;
                }
                writeln!(writer, "{}", line).unwrap();
                writer.flush().unwrap();
            }
        }
        break;
    }

    loop {
        match &*get_entry() {
            "" => {
                ;
            }
            "/quit" => {
                println!("Disconnecting...");
                writeln!(writer, "BYE").unwrap();
                writer.flush().unwrap();
                println!("Disconnected!");
                return ();
            }
            "/clients" => {
                writeln!(writer, "REQ CLIENTS").unwrap();
                writer.flush().unwrap();
            }
            line => {
                writeln!(writer, "MSG {}", line).unwrap();
                writer.flush().unwrap();
            }
        }
    }
}

fn exchange_with_server(stream: TcpStream) {
    let server = stream.peer_addr().unwrap();
    println!("Connected to {}", server);

    let stream_cpy = stream.try_clone().unwrap();
    let mut reader = BufReader::new(&stream_cpy);

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
        let input: String = String::from(receive!());
        let spliced_input: Vec<&str> = input.split(" ").collect();
        // if spliced_input[0] == "FROM" {
        //     println!("<{}>: {}", spliced_input[1], spliced_input[3]);
        //     continue;
        // }
        match spliced_input[0] {
            "FROM" => {
                println!("<{}>: {}", spliced_input[1], spliced_input[3]);
            }
            _ => {
                println!("{}", input);
            }
        }
        // println!("{}", input);
    })()
    {
        Ok(_) => {
            println!("Left?");
        }
        Err(_) => {
            println!(">>> Successfully left the room <<<");
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
            return;
        }
    }
}
