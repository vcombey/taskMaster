pub const HELP_START: &str = "
start <name>		Start a process
start <gname>:*		Start all processes in a group
start <name> <name>	Start multiple processes or groups
start all		Start all processes
";

pub const HELP_RESTART: &str = "
restart <name>		Restart a process
restart <gname>:*	Restart all processes in a group
restart <name> <name>	Restart multiple processes or groups
restart all		Restart all processes
Note: restart does not reread config files. For that, see reread and update.
";

pub const HELP_STOP: &str = "
stop <name>		Stop a process
stop <gname>:*		Stop all processes in a group
stop <name> <name>	Stop multiple processes or groups
stop all		Stop all processes
";

pub const HELP_RELOAD: &str = "
reload 		Restart the remote supervisord.
";

pub const HELP_STATUS: &str = "
status <name>		Get status for a single process
status <gname>:*	Get status for all processes in a group
status <name> <name>	Get status for multiple named processes
status			Get all process status info
";

pub const HELP_SHUTDOWN: &str = "
shutdown 	Shut the remote supervisord down.
";

pub const HELP_DISPLAY: &str = "
default commands (type help <topic>):
=====================================
start  restart   stop  reload  status    shutdown
";
