#![feature(type_ascription)]

use std::env;

pub mod client;
pub mod server;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() == 2 {
        ///////////////////////////////////////////////////////////////////////
        //                           Server opened                           //
        ///////////////////////////////////////////////////////////////////////
        println!("Opening server on port {}", args[1]);
        // serveur(args[1].clone());
        let mut serv = String::from("127.0.0.1:");
        serv.push_str(&args[1]);
        server::serveur(serv);
    } else if args.len() == 3 {
        ///////////////////////////////////////////////////////////////////////
        //                           Client opened                           //
        ///////////////////////////////////////////////////////////////////////
        println!("Client connecting on server {}:{}", args[1], args[2]);
        let mut serv = args[1].clone();
        serv.push(':');
        serv.push_str(&args[2]);
        client::client(serv);
    } else {
        println!("Usage: {} [server ip] port", args[0]);
    }
}
