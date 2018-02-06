use std::process::Command;
use std::fs::File;
use std::process::Child;
use std::sync::mpsc::{Receiver, Sender};
use std::time::Instant;
use std::os::unix::process::CommandExt;
use std::os::unix::process::ExitStatusExt;
use std::io::Error;
use std::thread::sleep;
use std::time::Duration;
use std::process::Stdio;

use nix::sys::stat::umask;
use nix::sys::stat::Mode;
use nix::sys::signal::kill;
use nix::sys::signal::Signal;
use nix::unistd::Pid;

use tm_mod::config::Config;
use tm_mod::config::Autorestart;
use tm_mod::cmd::Instruction;

type Message = String;
#[derive(Debug, PartialEq)]
enum State {
    RUNNING,
    BACKOFF,
    STOPPED,
    UNLAUNCHED,
    EXITED,
    KILLED,
}

#[derive(Debug)]
/// A process struct represent a thread which handle a process
pub struct Process {
    command: Command,
    config: Config,
    sender: Sender<String>,
    receiver: Receiver<(Instruction, Option<Config>)>,
    child: Option<Child>,
    state: State,
}

impl Process {
    /// Create a new Process with:
    /// a Config,
    /// a Receiver from the main thread,
    /// a Sender to the main thread.
    /// And set child to None and state to State::UNLAUNCHED
    pub fn new(
        config: Config,
        receiver: Receiver<(Instruction, Option<Config>)>,
        sender: Sender<String>,
    ) -> Self {
        Process {
            command: Command::new(config.argv.split(' ').next().unwrap()),
            config,
            receiver,
            sender,
            child: None,
            state: State::UNLAUNCHED,
        }
    }

    fn add_workingdir(&mut self) -> &mut Process {
        if let Some(ref string) = self.config.workingdir {
            self.command.current_dir(string);
        }
        self
    }

    fn add_args(&mut self) -> &mut Process {
        if self.config.argv.len() > 1 {
            let args: Vec<&str> = self.config.argv.split_whitespace().collect();
            self.command.args(&args[1..]);
        }
        self
    }

    fn add_stdout(&mut self) -> &mut Process {
        if let Some(ref string) = self.config.stdout {
            match File::create(string) {
                Ok(file) => {
                    self.command.stdout(file);
                }
                Err(e) => {
                    eprintln!("{}", e);
                    self.command.stdout(Stdio::null());
                }
            }
        } else {
            self.command.stdout(Stdio::null());
        }
        self
    }

    fn add_stderr(&mut self) -> &mut Process {
        if let Some(ref string) = self.config.stderr {
            match File::create(string) {
                Ok(file) => {
                    self.command.stderr(file);
                }
                Err(e) => {
                    eprintln!("{}", e);
                    self.command.stderr(Stdio::null());
                }
            }
        } else {
            self.command.stderr(Stdio::null());
        }
        self
    }

    fn add_env(&mut self) -> &mut Process {
        if let Some(ref vect) = self.config.env {
            let v: Vec<(String, String)> = vect.to_vec();
            self.command.envs(v);
        }
        self
    }

    #[cfg(not(target_os = "linux"))]
    fn add_umask(&mut self) -> &mut Process {
        let conf_umask = self.config.umask;

        let call_umask = move || -> Result<(), Error> {
            let mode = Mode::from_bits(conf_umask).unwrap();
            umask(mode);
            Ok(())
        };

        self.command.before_exec(call_umask);
        self
    }

    #[cfg(target_os = "linux")]
    fn add_umask(&mut self) -> &mut Process {
        let conf_umask = self.config.umask as u32;

        let call_umask = move || -> Result<(), Error> {
            let mode = Mode::from_bits(conf_umask).unwrap();
            umask(mode);
            Ok(())
        };

        self.command.before_exec(call_umask);
        self
    }

    fn spawn(&mut self) -> &mut Process {
        let child = self.command.spawn();
        if let Ok(child) = child {
            self.child = Some(child);
        }
        self
    }

    /// use the builder design pattern to spawn a process
    fn start(&mut self) -> &mut Process {
        self.add_args()
            .add_workingdir()
            .add_env()
            .add_stdout()
            .add_stderr()
            .add_umask()
            .spawn()
    }

    /// call try wait on the child if any, update status, and write info if exited
    fn try_wait(&mut self) -> (Option<i32>, Option<Signal>) {
        if let Some(mut child) = self.child.take() {
            match child.try_wait() {
                /* le program has ended */
                Ok(Some(exit_status)) => {
                    match exit_status.code() {
                        Some(exit_status_code) => {
                            if self.config.exitcodes.contains(&(i64::from(exit_status_code))) {
                                eprintln!(
                                    "INFO exited: '{}' (exit status {}; expected)",
                                    self.config.name, exit_status_code
                                );
                            } else {
                                eprintln!(
                                    "INFO exited: '{}' (exit status {}; unexpected)",
                                    self.config.name, exit_status_code
                                );
                            }
                            self.state = State::EXITED;
                            return (Some(exit_status_code), None);
                        }
                        None => {
                            if let Some(exit_signal) = exit_status.signal() {
                                let exit_signal = Signal::from_c_int(exit_signal).unwrap();
                                eprintln!(
                                    "INFO stopped: '{}' (terminated by {:?}) ",
                                    self.config.name, exit_signal
                                );
                                self.state = State::STOPPED;
                                return (None, Some(exit_signal));
                            }
                        }
                    }
                    //self.state = State::BACKOFF;
                }
                _ => {
                    self.child = Some(child);
                    self.state = State::RUNNING;
                    return (None, None);
                }
            }
        }
        (None, None)
    }

    /// try launch the programe one time
    /// return after starttime if the program is still running
    /// or return if the program has exited before starttime
    fn try_launch(&mut self) {
        self.start();
        if let Some(ref mut child) = self.child {
            let now = Instant::now();
            let mut i = 0;
            loop {
                if i == 0 {
                    eprintln!(
                        "INFO spawned: '{}' with pid {:?}",
                        self.config.name,
                        child.id()
                    );
                }
                i += 1;
                match child.try_wait() {
                    /* le program has ended */
                    Ok(Some(exit_status)) => {
                        let exit_status_code = exit_status.code().unwrap_or(1);
                        let nownow = Instant::now();
                        let duree = nownow.duration_since(now);

                        /* it is an unexpected ended */
                        if duree < self.config.starttime {
                            eprintln!(
                                "INFO exited: '{}' (exit status {}; hasn't lived enough)",
                                self.config.name, exit_status_code
                            );

                        /* it is an expected ended */
                        } else {
                            eprintln!(
                                "INFO exited: '{}' (exit status {};  has lived enough)",
                                self.config.name, exit_status_code
                            );
                        }
                        self.state = State::EXITED;
                        break;
                    }
                    /* le program has not ended yet : check the time*/
                    Ok(None) => {
                        let nownow = Instant::now();
                        let duree = nownow.duration_since(now);
                        if duree > self.config.starttime {
                            self.state = State::RUNNING;
                            eprintln!(
                                "INFO spawned: '{}' with pid {:?}",
                                self.config.name,
                                child.id()
                            );
                            break;
                        } else {
                            continue;
                        }
                    }
                    Err(e) => eprintln!("error attempting to wait: {}", e),
                }
            }
        }
    }

    /// call in loop try launch no more than startretries or
    /// until the program has started
    /// change state to state backoff if don't succeed launch the process
    fn try_execute(&mut self) -> Message {
        if self.state == State::RUNNING {
            return format!("{}: ERROR (already running)", self.config.name);
        }
        for _ in 0..self.config.startretries + 1 {
            //eprintln!("nb_try {}, startretries {}", nb_try, self.config.startretries);
            self.try_launch();
            if self.state == State::RUNNING {
                return format!("{}: RUNNING", self.config.name);
            }
        }
        self.state = State::BACKOFF;
        format!("{}: FATAL (too many try)", self.config.name)
    }

    /// try stoping the process by sending the stopsignal stated in the conf
    /// if the process isn't dead after stoptime send a SIGKILL to it
    fn stop(&mut self) -> Message {
        if let Some(ref mut child) = self.child {
            match kill(Pid::from_raw(child.id() as i32), self.config.stopsignal) {
                Ok(_) | Err(_) => {}
            }
        } else {
            eprintln!("{}: ERROR (not running)", self.config.name);
            return format!("{}: ERROR (not running)", self.config.name);
        }
        let now = Instant::now();
        loop {
            let nownow = Instant::now();
            let duree = nownow.duration_since(now);
            sleep(Duration::from_millis(10));

            self.try_wait();

            // stoped with stopsignal
            if self.state != State::RUNNING {
                return format!("{}: STOPPED", self.config.name);
            }

            // have to kill manualy
            if duree > self.config.stoptime {
                if let Some(ref mut child) = self.child {
                    match child.kill() {
                        Ok(_) => {}
                        Err(_) => eprintln!("{}: ERROR (not running)", self.config.name),
                    }
                }
                self.state = State::KILLED;
                return format!("{}: KILLED", self.config.name);
            }
        }
    }

    /// Send a formated string about the status of the process to the main thread
    fn status(&mut self) -> Message {
        format!("{}: {:?}", self.config.name, self.state)
    }

    fn handle_cmd(&mut self, cmd: Instruction, config: Option<Config>) {
        let message = match cmd {
            Instruction::STOP => self.stop(),
            Instruction::STATUS => self.status(),
            Instruction::START => self.try_execute(),
            Instruction::RESTART => format!("{}\n{}", self.stop(), self.try_execute()),
            Instruction::REREAD => match config {
                Some(config) => {
                    self.config = config;
                    String::from("Process locally updating")
                }
                None => String::from("No config sent"),
            },
            _ => String::from("unrecognised instruction"),
        };
        if self.sender.send(message).is_err() {
            eprintln!("process send to main thread failed");
        }
    }

    /// try receive Once and then loop forever : try receiving and waiting
    /// alternatively
    pub fn manage_program(&mut self) {
        if self.config.autostart {
            self.try_execute();
        }

        loop {
            if let Ok((ins, conf)) = self.receiver.try_recv() {
                eprintln!("INFO process '{}' receive {:?}", self.config.name, ins);
                if ins == Instruction::SHUTDOWN {
                    self.stop();
                    break;
                }
                self.handle_cmd(ins, conf);
            }
            let (stat, sig) = self.try_wait();
            match self.config.autorestart {
                Autorestart::TRUE => {
                    self.try_execute();
                }
                Autorestart::UNEXPECTED => match (stat, sig) {
                    (Some(exit_status), None) => {
                        if !self.config.exitcodes.contains(&i64::from(exit_status)) {
                            self.try_execute();
                        }
                    }
                    (None, Some(signal)) => {
                        if signal != Signal::SIGKILL && self.config.stopsignal != signal {
                            self.try_execute();
                        }
                    }
                    _ => {}
                },
                Autorestart::FALSE => {}
            };
        }
    }
}
