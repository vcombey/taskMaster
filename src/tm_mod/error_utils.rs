use std::error::Error;

pub fn print_err<T, E>(r: Result<T, E>)
    where E: Error {
    if let Err(e) = r {
        eprintln!("{}", e.description());
    }
}
