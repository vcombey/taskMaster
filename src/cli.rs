extern crate liner;

use cmd::Cmd;

use self::liner::Context;

const HELP_START : &'static str = "
start <name>		Start a process
start <gname>:*		Start all processes in a group
start <name> <name>	Start multiple processes or groups
start all		Start all processes
";

const HELP_RESTART : &'static str = "
restart <name>		Restart a process
restart <gname>:*	Restart all processes in a group
restart <name> <name>	Restart multiple processes or groups
restart all		Restart all processes
Note: restart does not reread config files. For that, see reread and update.
";

const HELP_STOP : &'static str = "
stop <name>		Stop a process
stop <gname>:*		Stop all processes in a group
stop <name> <name>	Stop multiple processes or groups
stop all		Stop all processes
";

const HELP_RELOAD : &'static str = "
reload 		Restart the remote supervisord.
";

const HELP_STATUS : &'static str = "
status <name>		Get status for a single process
status <gname>:*	Get status for all processes in a group
status <name> <name>	Get status for multiple named processes
status			Get all process status info
";

const HELP_SHUTDOWN : &'static str = "
shutdown 	Shut the remote supervisord down.
";

const HELP_DISPLAY : &'static str = "
default commands (type help <topic>):
=====================================
start  restart   stop  reload  status    shutdown
";

pub fn parse_cmd(line: &str) -> Option<(Cmd, Vec<&str>)> {
    let split: Vec<&str> = line.split(" ").collect();
    
    match split[0] {
        "help" => {
            let join = split[1..].join(" ");
            match &join[..] {
                "start" => println!("{}", HELP_START),
                "restart" => println!("{}", HELP_RESTART),
                "stop" => println!("{}", HELP_STOP),
                "reload" => println!("{}", HELP_RELOAD),
                "status" => println!("{}", HELP_STATUS),
                "shutdown" => println!("{}", HELP_SHUTDOWN),
                "" => println!("{}", HELP_DISPLAY),
                other => println!("*** No help on {}", other),
            }
            None
        },
        "stop" => Some((Cmd::STOP, split[1..].to_vec())),//.iter().map(|s| String::from(*s)).collect())),
        "start" => Some((Cmd::START, split[1..].to_vec())),
        "restart" => Some((Cmd::RESTART, split[1..].to_vec())),
        "reload" => Some((Cmd::RELOAD, split[1..].to_vec())),
        "shutdown" => Some((Cmd::SHUTDOWN, split[1..].to_vec())),
        "status" => Some((Cmd::STATUS, split[1..].to_vec())),
        _ => {
            println!("*** Unknown syntax: {:?}", line);
            None
        },
    }
}
