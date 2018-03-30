use common::get_time;
#[allow(unused_imports)]
use gtk;
use gtk::prelude::*;
#[allow(unused_imports)]
use gtk::{Builder, Button, Dialog, Entry, MessageDialog, TextView, Window};
use protocol::PROTOCOL;
use std;
use std::io::*;
use std::net::TcpStream;
#[allow(unused_imports)]
use std::thread;

fn get_chat(builder: &Builder) -> TextView {
    builder
        .get_object("txt_chat")
        .expect("Could not get txt_chat in client::engine::get_chat()")
}

fn send_message(stream: TcpStream, msg: String) {
    let mut writer = BufWriter::new(&stream);
    writeln!(writer, "MSG {}", msg).unwrap();
    writer.flush().unwrap();
}

fn write_to_chat(builder: &Builder, msg: String) {
    let chat = get_chat(builder);
    chat.get_buffer().unwrap().insert(
        &mut chat.get_buffer().unwrap().get_end_iter(),
        &format!("{} {}\n", get_time(), msg),
    );
}

fn get_entry() -> String {
    let mut buf = String::new();
    stdin().read_line(&mut buf).unwrap();
    buf.replace("\n", "").replace("\r", "")
}

fn get_name() -> String {
    'mainloop: loop {
        println!("{}", "Please enter your name:");
        let mut name = &*get_entry();
        name = name.trim();
        if name.len() > 20 {
            println!(
                "{}",
                "Nickname too long, it must be at most 20 characters long"
            );
            continue;
        }
        for c in name.chars() {
            if !c.is_ascii() {
                println!(
                    "{}{}{}",
                    "Character ",
                    &format!("{}", c),
                    " is not an ASCII character."
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
                    println!("{}", "Cannot use whitespace in name");
                    continue;
                }
                return String::from(name);
            }
        }
    }
}

fn exchange_with_server(stream: TcpStream, builder: Builder, nick: String) {
    let mut nick_cpy = nick.clone();
    let server = stream.peer_addr().unwrap();
    println!("Connected to {}", server);

    let stream_cpy = stream.try_clone().unwrap();
    let mut reader = BufReader::new(&stream_cpy);
    let mut writer = BufWriter::new(&stream_cpy);

    // let txt_chat = get_chat(&builder);

    macro_rules! receive {
        () => {{
            let mut line = String::new();
            match reader.read_line(&mut line) {
                Ok(len) => {
                    if len == 0 {
                        // Reader is at EOF. Could use ErrorKind::UnexpectedEOF,
                        // but still unstable.
                        let ret = std::io::Error::new(std::io::ErrorKind::Other, "test");
                        return Err(ret): std::result::Result<&str, std::io::Error>;
                    }
                    line.pop();
                }
                Err(e) => {
                    return Err(e);
                }
            };
            line
        }};
    }

    // entrée du nom d'utilisateur
    let _nick : String = match (|| loop {
        let name : String;
        if !nick_cpy.is_empty() {
            println!("Connecting with username {}", nick_cpy);
            writeln!(writer, "PROT {} CONNECT USER {}", PROTOCOL, nick_cpy).unwrap();
            writer.flush().unwrap();
            name = nick_cpy;
            nick_cpy = String::new();
        } else {
            println!("Connecting as new user");
            writeln!(writer, "PROT {} CONNECT NEW", PROTOCOL).unwrap();
            writer.flush().unwrap();
            let answer = receive!();
            if answer != "NAME REQ" {
                return Err(Error::new(ErrorKind::Other, answer));
            }
            name = get_name();
            writeln!(writer, "NAME {}", name).unwrap();
            writer.flush().unwrap();
        }
        match receive!().as_str() {
            "NAME OK" => {
                write_to_chat(&builder, String::from("NAME OK"));
                name
            }

            "NAME FAILURE" => {
                write_to_chat(&builder, String::from("Username refused by server."));
                continue;
            }
            answer => {
                write_to_chat(&builder, String::from(format!("Server answered: {}", answer)));
                return Err(Error::new(ErrorKind::Other, answer));
            }
        };
    })(){
        Ok(name) => String::from(name),
        Err(_) => {
            write_to_chat(&builder, String::from(">>> Login successful <<<"));
            nick
        }
    };

    loop {}
}

pub fn client(builder: Builder, server_address: String, nick: String) {
    {
        let txt_chat = get_chat(&builder);
        txt_chat.get_buffer().unwrap().insert(
            &mut txt_chat.get_buffer().unwrap().get_end_iter(),
            &format!(
                "Trying to connect to the server {} as {}\n",
                server_address, nick
            ),
        );
    }
    match TcpStream::connect(server_address.clone()) {
        Ok(stream) => {
            exchange_with_server(stream, builder, nick);
        }
        Err(e) => {
            let txt_chat = get_chat(&builder);
            txt_chat.get_buffer().unwrap().insert(
                &mut txt_chat.get_buffer().unwrap().get_end_iter(),
                &format!("{} Connection to server failed: {}", get_time(), e),
            );
            return;
        }
    }
}
