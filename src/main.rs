extern crate sqlite;

mod data;

fn main() {
    println!("  _________.__       .__                        ____.                    .__\n /   _____/|__| ____ |  |   ____               |    | ____   ____   ____ |__| _________.__. ______\n \\_____  \\ |  |/ __ \\|  |  /  _ \\   ______     |    |/ __ \\ /    \\ /    \\|  |/  ___<   |  |/  ___/\n /        \\|  \\  ___/|  |_(  <_> ) /_____/ /\\__|    \\  ___/|   |  \\   |  \\  |\\___ \\ \\___  |\\___ \\\n/_______  /|__|\\___  >____/\\____/          \\________|\\___  >___|  /___|  /__/____  >/ ____/____  >\n        \\/         \\/                                    \\/     \\/     \\/        \\/ \\/         \\/");

    let mut connection = sqlite::Connection::open("./demo.db").unwrap();

    match data::history::History::new(&mut connection) {
        Ok(t) => {
            println!("Oki");
        },
        Err(e) => {
            println!("{:?}", e);
        }
    }
}