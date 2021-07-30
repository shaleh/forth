#![feature(iter_intersperse)]

use std::io::{self, Write};

mod forth;

use forth::Forth;

fn main() {
    let mut forth = Forth::new();

    loop {
        let mut input = String::new();

        print!("> ");
        io::stdout().flush().unwrap();

        match io::stdin().read_line(&mut input) {
            Ok(count) if count == 0 => {
                break;
            }
            Ok(_) => match forth.eval(&input) {
                Ok(None) => {
                    break;
                }
                Ok(Some(())) => {}
                Err(msg) => {
                    println!("Error: {}", msg);
                }
            },
            Err(msg) => {
                println!("Error: {}", msg);
                break;
            }
        }
    }
}
