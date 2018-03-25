extern crate chrono;
extern crate colored;
extern crate term_size;
use std;
use std::io::*;
use std::net::TcpStream;
use std::thread;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use self::colored::*;
use self::chrono::Local;

/*

0.1   [X]
1.1   [X]
1.2   [ ]
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
4.2.2 [X]

 */

// TODO: Implement requests 1.2

fn hash_name(name: &str) -> usize {
    let mut s = DefaultHasher::new();
    let name = String::from(name);
    name.hash(&mut s);
    s.finish() as usize
}

fn get_entry() -> String {
    let mut buf = String::new();
    stdin().read_line(&mut buf).unwrap();
    buf.replace("\n", "").replace("\r", "")
}

fn get_name() -> String {
    'mainloop: loop {
        println!("{}", "Please enter your name:".yellow().dimmed());
        let mut name = &*get_entry();
        name = name.trim();
        if name.len() > 20 {
            println!(
                "{}",
                "Nickname too long, it must be at most 20 characters long".red()
            );
            continue;
        }
        for c in name.chars() {
            if !c.is_ascii() {
                println!(
                    "{}{}{}",
                    "Character ".red(),
                    &format!("{}", c).green(),
                    " is not an ASCII character.".red()
                );
                continue 'mainloop;
            }
        }
        match name {
            "" => {
                continue;
            }
            _ => {
                let spliced_name: Vec<&str> = name.split_whitespace().collect();
                if spliced_name.len() != 1 {
                    println!("{}", "Cannot use whitespace in name".red());
                    continue;
                }
                return String::from(name);
            }
        }
    }
}

fn write_to_server(stream: TcpStream) {
    let mut writer = BufWriter::new(&stream);

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
                if line.len() > 2000 {
                    println!(
                        "{}",
                        "Cannot send a message longer than 2000 characters".bright_red()
                    );
                    continue;
                }
                writeln!(writer, "MSG {}", line).unwrap();
                writer.flush().unwrap();
            }
        }
    }
}

fn exchange_with_server(stream: TcpStream) {
    let server = stream.peer_addr().unwrap();
    println!("Connected to {}", server);

    #[allow(non_snake_case)]
    let COLORS: Vec<&str> = vec![
        // "black",
        "red",
        "green",
        "yellow",
        "blue",
        "magenta",
        "cyan",
        // "white",
    ];

    let stream_cpy = stream.try_clone().unwrap();
    let mut reader = BufReader::new(&stream_cpy);
    let mut writer = BufWriter::new(&stream_cpy);

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

    // entrée du nom d'utilisateur
    writeln!(writer, "PROT {} CONNECT NEW", ::PROTOCOL).unwrap();
    writer.flush().unwrap();
    let _name: String = match (|| loop {
        let _answer = receive!();
        if _answer != "NAME REQ" {
            return Err(Error::new(ErrorKind::Other, _answer));
        }
        let nick = get_name();
        writeln!(writer, "NAME {}", nick).unwrap();
        writer.flush().unwrap();
        match receive!().as_str() {
            "NAME OK" => {
                println!("NAME OK");
                nick
            }

            "NAME FAILURE" => {
                println!("{}", "Username refused by server.".red());
                continue;
            }
            answer => {
                println!("{}{}", "Server answered: ".yellow().dimmed(), answer);
                return Err(Error::new(ErrorKind::Other, answer));
            }
        };
    })()
    {
        Ok(name) => String::from(name),
        Err(_) => {
            println!("{}", ">>> Login successful".green());
            String::new()
        }
    };

    thread::spawn(move || {
        write_to_server(stream.try_clone().unwrap());
    });

    match (|| loop {
        let input: String = String::from(receive!());
        let spliced_input: Vec<&str> = input.split(" ").collect();
        match spliced_input[0] {
            "BAD" => {
                println!("{}", "Bad request from client".red());
            }
            "WELCOME" => {
                println!("{}", ">>> Login Successful <<<".green());
                println!("Type /clients to get the list of users connected");
                println!("Type /quit to disconnect and quit");
            }
            "FROM" => {
                let date = Local::now();
                let name = String::from(spliced_input[1]);
                let mut first_line = true;

                // Hashing name for color
                let mut s = DefaultHasher::new();
                name.hash(&mut s);
                let name_hash: usize = (s.finish() as usize) % COLORS.len();

                // Formatting name
                let mut name = String::new();
                for _i in 0..(20 - spliced_input[1].to_string().len()) {
                    name.push(' ');
                }
                name.push('<');
                name.push_str(spliced_input[1]);
                name.push('>');

                // Display message with soft-wrap
                if let Some((w, _)) = term_size::dimensions() {
                    let mut msg = String::new();
                    let w = w - 34;

                    // format message
                    for mut i in 3..spliced_input.len() {
                        if w > msg.len() + spliced_input[i].len() + 1 {
                            msg.push(' ');
                            msg.push_str(spliced_input[i]);
                        } else {
                            if first_line == true {
                                println!(
                                    "{}{}{}{}",
                                    date.format("[%H:%M:%S]").to_string().dimmed(),
                                    name.color(COLORS[name_hash]),
                                    " |".green(),
                                    msg.yellow().dimmed()
                                );
                                first_line = false;
                            } else {
                                println!(
                                    "{}{}",
                                    "                                 |".green(),
                                    msg.yellow().dimmed()
                                );
                            }
                            msg = String::new();
                            #[allow(unused_assignments)]
                            #[allow(unused_assignments)]
                            i = i - 1;
                        }
                    }

                    if first_line == true {
                        println!(
                            "{}{}{}{}",
                            date.format("[%H:%M:%S]").to_string().dimmed(),
                            name.color(COLORS[name_hash]),
                            " |".green(),
                            msg.yellow().dimmed()
                        );
                    } else {
                        println!(
                            "{}{}",
                            "                                 |".green(),
                            msg.yellow().dimmed()
                        );
                    }
                } else {
                    let mut msg = String::new();
                    for i in 3..spliced_input.len() {
                        msg.push_str(" ");
                        msg.push_str(spliced_input[i]);
                    }
                    println!(
                        "{} {}{}",
                        date.format("[%H:%M:%S]").to_string().dimmed(),
                        name.color(COLORS[name_hash]),
                        msg.yellow().dimmed()
                    );
                }
            }
            "BYE" => {
                return Ok("Ok");
            }
            "LIST" => {
                println!(
                    "{}{}{}",
                    ">>>> LIST OF CLIENTS CONNECTED (".bold().yellow(),
                    spliced_input[2],
                    ") <<<<".bold().yellow()
                );
                for i in 3..spliced_input.len() {
                    println!("\t\t{}", spliced_input[i]);
                }
            }
            "JOIN" => {
                let date = Local::now();
                let name_hash: usize = hash_name(spliced_input[1].clone()) % COLORS.len();

                println!(
                    "{}{}{}{}",
                    date.format("[%H:%M:%S]").to_string().dimmed(),
                    "               ------>   ".green(),
                    spliced_input[1].color(COLORS[name_hash]),
                    " has joined".green()
                )
            }
            "LOGOUT" => {
                let date = Local::now();
                let name_hash: usize = hash_name(spliced_input[1].clone()) % COLORS.len();

                println!(
                    "{}{}{}{}",
                    date.format("[%H:%M:%S]").to_string().dimmed(),
                    "               <------   ".red(),
                    spliced_input[1].color(COLORS[name_hash]),
                    " has left".red()
                )
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
        Err(_) => {
            println!("{}", "Error: Connection with server lost".red());
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
