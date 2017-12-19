use std::io;
use std::io::Write;
use std::env;

fn main() {
	loop {
        let args: Vec<String> = env::args().collect();
        println!("{:?}", args);
		print!("task_master> ");
		io::stdout().flush().unwrap();
		let mut guess = String::new();

		io::stdin().read_line(&mut guess)
			.expect("Failed to read line");

		println!("You wrote: {}", guess);
	}
}
