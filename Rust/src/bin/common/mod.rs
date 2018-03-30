extern crate chrono;

use self::chrono::Local;

pub fn get_time() -> String {
    let date = Local::now();
    date.format("[%H:%M:%S]").to_string()
}
