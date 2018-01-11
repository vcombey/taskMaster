#[derive(Serialize, PartialEq, Deserialize, Debug)]
pub enum Cmd {
    START,
    RESTART,
    STOP,
    RELOAD,
    STATUS,
    SHUTDOWN,
}
