use std::env;


use rasdf::AsdfBase;

fn main() {
    let mut argiter = env::args();
    let prog = argiter.next().unwrap_or(String::new());
    let cmnd = argiter.next().unwrap_or(String::new());
    let args: Vec<String> = argiter.collect();

    println!("{}: {}, {:?}", prog, cmnd, args);
}
