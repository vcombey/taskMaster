pub mod service {
    use process::process::Process;
    use std::collections::HashMap;
    pub struct Service {
        name: String,
        process: HashMap<String, Process>,
    }

    impl Service {
        pub fn new(name: String) {
        }
    }
}
