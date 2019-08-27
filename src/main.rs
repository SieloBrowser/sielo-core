extern crate sqlite;

mod data;
use data::db::{TableProvider, FieldType, FieldParameter};

fn main() {
    println!("  _________.__       .__                        ____.                    .__\n /   _____/|__| ____ |  |   ____               |    | ____   ____   ____ |__| _________.__. ______\n \\_____  \\ |  |/ __ \\|  |  /  _ \\   ______     |    |/ __ \\ /    \\ /    \\|  |/  ___<   |  |/  ___/\n /        \\|  \\  ___/|  |_(  <_> ) /_____/ /\\__|    \\  ___/|   |  \\   |  \\  |\\___ \\ \\___  |\\___ \\\n/_______  /|__|\\___  >____/\\____/          \\________|\\___  >___|  /___|  /__/____  >/ ____/____  >\n        \\/         \\/                                    \\/     \\/     \\/        \\/ \\/         \\/");

    let mut connection = data::db::sqlite::SQLite::new("./demo.db").ok().unwrap();
    //let mut connection = data::db::sqlite::SQLite::new(":memory:").ok().unwrap();

    match connection.use_table("history", &[
        ("id", &FieldType::Integer, &[FieldParameter::AutoIncrement]),
        ("mime_type", &FieldType::Text, &[FieldParameter::Default(String::from("sielo/unknown"))]),
        ("url", &FieldType::Text, &[FieldParameter::NoNull]),
        ("date", &FieldType::Integer, &[FieldParameter::Default(String::from("0"))]),
        ("title", &FieldType::Text, &[FieldParameter::Default(String::new())]),
        ("favicon", &FieldType::Blob, &[]),
        ("parent", &FieldType::Integer, &[]),
        ("children", &FieldType::Blob, &[]),
        ("profile", &FieldType::Text, &[]),
    ], false, false) {
        Ok(t) => (),
        Err(e) => {
            println!("{:?}", e);
        }
    }

    /*let mut connection = sqlite::Connection::open("./demo.db").unwrap();

    match data::history::History::new(&mut connection) {
        Ok(t) => {
            println!("Oki");
        },
        Err(e) => {
            println!("{:?}", e);
        }
    }*/
}
