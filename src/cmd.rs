#[derive(Debug,PartialEq)]
pub enum Cmd {
    START,
    RESTART,
    STOP,
    RELOAD,
    STATUS,
    SHUTDOWN,
}
