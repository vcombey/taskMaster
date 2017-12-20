use std::{time,thread};

fn main()
{
    thread::sleep(time::Duration::from_secs(2));
    println!("Hello world");
}
