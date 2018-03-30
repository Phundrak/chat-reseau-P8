use gio;
use gio::prelude::*;
use gtk;
use gtk::prelude::*;
use gtk::{ApplicationWindow, Builder, Button, Dialog, Entry, MessageDialog, TextView, Window};
use std::env::args;

pub mod engine;

macro_rules! clone {
    (@param _) => ( _ );
    (@param $x:ident) => ( $x );
    ($($n:ident),+ => move || $body:expr) => (
        {
            $( let $n = $n.clone(); )+
                move || $body
        }
    );
    ($($n:ident),+ => move |$($p:tt),+| $body:expr) => (
        {
            $( let $n = $n.clone(); )+
                move |$(clone!(@param $p),)+| $body
        }
    );
}

fn connect_ui(button: &Button, builder: &Builder) {
    let diag: Dialog = builder
        .get_object("diag_login")
        .expect("Couldn’t get diag_login");
    let entry_addr: Entry = builder
        .get_object("entry_login_addr")
        .expect("Couldn’t get entry_login_add");
    let entry_port: Entry = builder
        .get_object("entry_login_port")
        .expect("Couldn’t get entry_login_port");
    let entry_nick: Entry = builder
        .get_object("entry_login_nick")
        .expect("Couldn’t get entry_login_nick");
    let btn_ok: Button = builder
        .get_object("btn_login_ok")
        .expect("Couldn’t get btn_login_ok");
    let entry_chat: Entry = builder
        .get_object("entry_chat")
        .expect("Couldn’t get entry_chat");
    let txt_clients: TextView = builder
        .get_object("txt_clients")
        .expect("Couldn’t get txt_clients");

    btn_ok.connect_clicked(
        clone!(diag, entry_addr, entry_port, entry_nick => move |_| {
            let addr = &format!(
                "{}:{}",
                entry_addr.get_buffer().get_text(),
                if entry_port.get_buffer().get_text() != "" {
                    entry_port.get_buffer().get_text()
                } else {
                    String::from("2222")
                });
            entry_chat.set_text(
                if entry_addr.get_buffer().get_text() == "" {
                    ""
                } else {
                    addr
                });
            txt_clients.get_buffer().unwrap().set_text(&entry_nick.get_buffer().get_text());
            diag.hide();
        }),
    );

    if let Some(window) = button
        .get_toplevel()
        .and_then(|w| w.downcast::<Window>().ok())
    {
        diag.set_transient_for(Some(&window));
    }

    diag.show();
    diag.run();
    diag.hide();
}

fn disconnect(button: &Button, builder: &Builder) {
    let diag: MessageDialog = builder
        .get_object("window_logout")
        .expect("Couldn’t get window_logout");

    if let Some(window) = button
        .get_toplevel()
        .and_then(|w| w.downcast::<Window>().ok())
    {
        diag.set_transient_for(Some(&window));
    }
    diag.show();
    diag.run();
    diag.hide();
}

fn build_ui(application: &gtk::Application) {
    println!(
        "GTK Version: {}.{}",
        gtk::get_major_version(),
        gtk::get_minor_version()
    );
    let glade_src = include_str!("client_gui.glade");
    let builder = Builder::new_from_string(glade_src);

    let window_main: ApplicationWindow = builder
        .get_object("window_main")
        .expect("Couldn't get window_main");
    window_main.set_application(application);

    let entry_chat: Entry = builder
        .get_object("entry_chat")
        .expect("Couldn’t get entry_chat");
    let txt_clients: TextView = builder
        .get_object("txt_clients")
        .expect("Couldn’t get txt_clients");

    let btn_connect: Button = builder
        .get_object("btn_connect")
        .expect("Couldn't get connect btn_connect");
    btn_connect.connect_clicked(clone!(builder => move |x| {
        connect_ui(x, &builder);
        let serv_addr : String = String::from(entry_chat.get_text().unwrap());
        let buf : gtk::TextBuffer = txt_clients.get_buffer().unwrap();
        let nick : String = String::from(buf.get_text(
            &buf.get_start_iter(),
            &buf.get_end_iter(),
            false)
                                         .unwrap());
        entry_chat.set_text("");
        txt_clients.get_buffer().unwrap().set_text("");
        println!("Server address: {}", serv_addr);
        println!("Nick: {}", nick);
        if serv_addr != "" {
            // client(builder.clone(), serv_addr, nick);
            engine::client(builder.clone(), serv_addr, nick);
        }

    }));

    let btn_disconnect: Button = builder
        .get_object("btn_disconnect")
        .expect("Couldn’t get disconnect btn_disconnect");
    btn_disconnect.connect_clicked(clone!(builder => move |x| {
        disconnect(x, &builder);
    }));

    window_main.show_all();
}

pub fn main() {
    let application = gtk::Application::new("com.github.gtktest", gio::ApplicationFlags::empty())
        .expect("Initialization failed...");

    application.connect_startup(move |app| {
        build_ui(app);
    });
    application.connect_activate(|_| {});

    application.run(&args().collect::<Vec<_>>());
}
