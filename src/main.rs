use std::fs;
use std::process;

fn die(msg: &str) {
    eprintln!("Fatal: {}", msg);
    process::exit(1);
}

fn main() {

    let source = std::env::args().nth(1).expect("No source given");
    let dest = std::env::args().nth(2).expect("No dest given");

    let doesexist = fs::exists(&source);
    match doesexist {
        Ok(true) => { println!("source exists"); },
        Ok(false) => { die("source does not exist"); },
        Err(_) => { die("error attempting to check source file existence"); }
    }

    println!("source: {:?}, dest: {:?}", source, dest);
}
