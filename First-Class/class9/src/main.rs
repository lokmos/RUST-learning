use std::thread::spawn;
use std::sync::Arc;

fn main() {
    let s = Arc::new("Hello, world");
    let s1 = s.clone();

    spawn(move || {
        println!("s: {:?}", s);
    });

    println!("s: {:?}", s1);
}