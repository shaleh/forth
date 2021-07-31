#![feature(iter_intersperse)]

use std::io::{self, Write};

mod forth;

use forth::{Forth, ForthError};

fn main() {
    let mut forth = Forth::new();

    loop {
        let mut input = String::new();

        print!("> ");
        io::stdout().flush().unwrap();

        match io::stdin().read_line(&mut input) {
            Ok(0) => {
                break;
            }
            Ok(_) => match forth.eval(&input) {
                Ok(result) => match result {
                    Some(value) => {
                        println!("{} Ok", value);
                    }
                    None => {
                        println!(" Ok");
                    }
                },
                Err(ForthError::UserQuit) => {
                    break;
                }
                Err(msg) => {
                    println!("? Error: {}", msg);
                }
            },
            Err(msg) => {
                println!("Error: {}", msg);
                break;
            }
        }
    }
}
