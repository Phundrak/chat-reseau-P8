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

// fn get_time() -> String {
//     let date = Local::now();
//     date.format("[%H:%M:%S]").to_string()
// }

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
        &format!("{} {}\n", get_time(), msg)
    );

}

fn receive_from_server(stream: TcpStream) -> std::result::Result<String, Error> {
    let mut reader = BufReader::new(&stream);
    let mut line = String::new();
    match reader.read_line(&mut line) {
        Ok(len) => {
            if len == 0 {
                // Reader is at EOF.
                let ret = Error::new(ErrorKind::UnexpectedEof, "test");
                return Err(ret);
            }
            line.pop();
        }
        Err(e) => {
            return Err(e);
        }
    };
    Ok(line)
}

fn exchange_with_server(stream: TcpStream, builder: Builder, nick: String) {
    let server = stream.peer_addr().unwrap();
    let txt_chat: TextView = builder
        .get_object("txt_chat")
        .expect("Couldn’t get txt_chat");
    txt_chat.get_buffer().unwrap().insert(
        &mut txt_chat.get_buffer().unwrap().get_end_iter(),
        &format!("Connected to server {} !\n", server),
    );

    let stream_cpy = stream.try_clone().unwrap();
    let mut writer = BufWriter::new(&stream_cpy);

    'nameloop: loop {
        if nick == "" {
            writeln!(writer, "PROT {} CONNECT USER {}", PROTOCOL, nick).unwrap();
            writer.flush().unwrap();
        } else {
            writeln!(writer, "PROT {} CONNECT NEW", PROTOCOL).unwrap();
            writer.flush().unwrap();
        }
        let answer = receive_from_server(stream.try_clone().unwrap()).unwrap();
        if answer.as_str() == "NAME OK" {
            if receive_from_server(stream.try_clone().unwrap()).unwrap().as_str() == "WELCOME" {
                write_to_chat(&builder, "Connected to server!".to_string());
                break 'nameloop;
            }
        }
    }

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
