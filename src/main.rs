use std::io;
use std::io::Write;

fn main() {
    loop {
        print!("$> ");
        match io::stdout().flush() {
            Ok(a) => a,
            Err(e) => panic!("try to flush but error occurs {:?}", e),
        }
        let mut guess = String::new();

        io::stdin().read_line(&mut guess)
            .expect("Failed to read line");

        println!("You writed: {}", guess);
    }
}
