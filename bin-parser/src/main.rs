#![allow(unused)]

use datatbls::parser;

fn main() {
    if let Err(err) = parser::run() {
        panic!("{err}");
    }
}
