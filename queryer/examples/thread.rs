use std::{
    sync::{Arc, Mutex},
    thread,
};

fn arc_mutext() {
    let a = Arc::new(Mutex::new(1));
    let b = a.clone();
    let c = a.clone();
    let hander = thread::spawn(move || {
        let mut g = b.lock().unwrap();
        *g += 1;
        println!("b is: {:?}", b);
    });
    {
        let mut g = c.lock().unwrap();
        *g += 1;
        println!("c is: {:?}", c);
    }
    hander.join().unwrap();
    println!("a is: {:?}", a);
}

fn main() {
    arc_mutext();
}
