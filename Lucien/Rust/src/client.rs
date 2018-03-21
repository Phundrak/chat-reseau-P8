extern crate chrono;
extern crate colored;
use std;
use std::io::*;
use std::net::TcpStream;
use std::thread;
use self::colored::*;
use self::chrono::Local;

// static leave_msg: &str = "BYE";

fn get_entry() -> String {
    let mut buf = String::new();

    stdin().read_line(&mut buf).unwrap();
    buf.replace("\n", "").replace("\r", "")
}

fn get_name(writer: &mut BufWriter<&TcpStream>) {
    loop {
        let mut line = &*get_entry();
        line = line.trim();
        match line {
            "" => {
                continue;
            }
            "/quit" => {
                println!("Disconnecting...");
                writeln!(writer, "BYE").unwrap();
                writer.flush().unwrap();
                return ();
            }
            line => {
                let line_str: String = String::from(line);
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
        return;
    }
}

fn write_to_server(stream: TcpStream) {
    let mut writer = BufWriter::new(&stream);

    // entrée du nom d'utilisateur
    get_name(&mut writer);

    loop {
        let line = &*get_entry();
        line.trim();
        match line {
            "" => {}
            "/quit" => {
                writeln!(writer, "BYE").unwrap();
                writer.flush().unwrap();
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

    macro_rules! receive {
        () => ({
            let mut line = String::new();
            match reader.read_line(&mut line) {
                Ok(len) => {
                    if len == 0 {
                        // Reader is at EOF. Could use ErrorKind::UnexpectedEOF,
                        // but still unstable.
                        let ret = std::io::Error::new(
                            std::io::ErrorKind::Other, "test");
                        return
                            Err(ret): std::result::Result<&str,std::io::Error>;
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
        match spliced_input[0] {
            "WELCOME" => {
                println!("{}", "Successfully connected to the server.".green());
                println!("Type /clients to get the list of users connected");
                println!("Type /quit to disconnect and quit");
            }
            "FROM" => {
                let date = Local::now();
                let mut msg = String::new();
                for i in 3..spliced_input.len() {
                    msg.push_str(" ");
                    msg.push_str(spliced_input[i]);
                }
                println!(
                    "{} <{}>:{}",
                    date.format("[%H:%M:%S]").to_string().blue(),
                    spliced_input[1].red(),
                    msg
                );
            }
            "BYE" => {
                return Ok("Ok");
            }
            "LIST" => {
                println!("{}", ">>>> LIST OF CLIENTS CONNECTED <<<<".bold().yellow());
                for i in 2..spliced_input.len() {
                    println!("\t\t{}", spliced_input[i]);
                }
            }
            "JOIN" => {
                println!(
                    "{}{}{}",
                    "-------> ".green(),
                    spliced_input[1],
                    " has joined".green()
                );
            }
            "LOGOUT" => {
                println!(
                    "{}{}{}",
                    "<------- ".red(),
                    spliced_input[1],
                    " has left".red()
                );
            }
            _ => {
                println!("{}", input);
            }
        }
        // println!("{}", input);
    })()
    {
        Ok(_) => {
            println!("{}", ">>> Logout successful <<<".green());
        }
        Err(e) => {
            println!("{}: {}", "Error: Function terminated too early".red(), e);
        }
    }
}

pub fn client(server_address: String) {
    println!("Trying to connect to the server...");
    match TcpStream::connect(server_address) {
        Ok(stream) => {
            exchange_with_server(stream);
        }
        Err(e) => {
            println!("{} {}", "Connection to server failed:".red(), e);
            return;
        }
    }
}
