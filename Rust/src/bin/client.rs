#![feature(type_ascription)]
#![feature(stmt_expr_attributes)]

extern crate gio;
extern crate gtk;

mod client_gui;
pub mod protocol;
pub mod common;

#[cfg(feature = "gtk_3_10")]
fn main() {
    client_gui::main();
}

#[cfg(not(feature = "gtk_3_10"))]
fn main() {
    println!("This example requires GTK 3.10 or later");
    println!("Did you forget to build or run with `--features gtk_3_10`?");
}
