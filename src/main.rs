#[allow(unused_imports)]
use std::io;
#[allow(unused_imports)]
use std::io::Write;
#[allow(unused_imports)]
use std::env;
#[allow(unused_imports)]
use std::fs::File;
#[allow(unused_imports)]
use std::io::prelude::*;
extern crate yaml_rust;
#[allow(unused_imports)]
use yaml_rust::{Yaml,YamlLoader, YamlEmitter};
#[allow(unused_imports)]
use std::process::Command;
//extern crate linked_hash_map;
//use linked_hash_map::LinkedHashMap;



fn exec_command_test() {
    
    println!("{:?}", env::current_dir().unwrap());
   /* Command::new("./wait_and_print").spawn().unwrap();
    Command::new("ls").arg("-l").spawn().unwrap();
    let mut child = Command::new("cat").spawn().unwrap();

    child.wait().unwrap();
    */
}

fn parse_argv (args: &[String]) -> (&str, &str)
{
    if args.len() < 3 {
        panic!("not enough arguments");
    }
    let option = &args[1];
    if option != "-c" {
        panic!("unknown option");
    }
    let filename = &args[2];

    (option, filename)
}

struct Process {
    command: Command,
    argv: String,
    workingdir: Option<String>,
    autostart: bool,
    env: Option<Vec<(String, String)>>,
    stdout: Option<String>,
    stderr: Option<String>,
}
    /*
    umask: i8,
    autorestart: i8,
    exitcodes: Vec<i8>,
    startretries: i8,
    starttime: i8,
    stopsignal: i8,
    stoptime: i8,
    */

impl Process {
    fn new(argv: String, 
           workingdir: Option<&str>,
           autostart: Option<bool>,
           env: Option<Vec<(String, String)>>,
           stdout: Option<&str>,
           stderr: Option<&str>
          ) -> Process {
        Process {
            command: Command::new(argv.split(" ").next().unwrap()),
            argv,
            workingdir: match workingdir {
                Some(slice) => Some(String::from(slice)),
                None => None,
            },
            autostart: match autostart {
                Some(value) => value,
                None => true,
            },
            env,
            stdout: match stdout {
                Some(slice) => Some(String::from(slice)),
                None => None,
            },
            stderr: match stderr {
                Some(slice) => Some(String::from(slice)),
                None => None,
            },
            /*
               umask: i8,
               stderr: File, //
               env: Option<Vec<(String, String)>>, //
               autorestart: i8,
               exitcodes: Vec<i8>,
               startretries: i8,
               starttime: i8,
               stopsignal: i8,
               stoptime: i8,
               */
        }
    }
    fn add_workingdir(mut self) -> Self {
        if let Some(ref string) = self.workingdir {
            self.command.env("PWD", string);
        }
        self
    }
    fn add_args(mut self) -> Self {
        if self.argv.len() > 1 {
            let args: Vec<&str> = self.argv.split(" ").collect();
            self.command.args(&args[1..]);
        }
        self
    }
    fn add_stdout(mut self) -> Self {
        if let Some(ref string) = self.stdout {
            match File::open(string) {
                Ok(file) => {self.command.stdout(file);},
                Err(e) => println!("error{:?}", e),
            }
        }
        self
    }
    fn add_stderr(mut self) -> Self {
        if let Some(ref string) = self.stderr {
            match File::open(string) {
                Ok(file) => {self.command.stderr(file);},
                Err(e) => println!("error{:?}", e),
            }
        }
        self
    }
    fn add_env(mut self) -> Self {
        //self.command.envs(self.env.unwrap());
        if let Some(ref vect) = self.env {
            let v: Vec<(String, String)> = vect.to_vec();
            self.command.envs(v);
        }
        self
    }
    fn spawn(&mut self) {
        self.command.spawn();
    }
}

fn exec_command (name: &Yaml, config: &Yaml) {
    //println!("name: {:#?} cmd: {:#?}", name, config);
    //println!("{:#?}", config["cmd"]);
    let cmd = (&config["cmd"]).as_str().unwrap();

    let working_dir = (&config["workingdir"]).as_str();
    let autostart = (&config["autostart"]).as_bool();
    let env: Option<Vec<(String, String)>> = match (&config["env"]).as_hash() {
        Some(hash) => { Some(hash.iter()
                             .map(|(name, cmd)| {
                                 (String::from(name.as_str().unwrap()), 
                                  String::from(cmd.as_str().unwrap()))
                             }) //TODO: gerer les nombre
                             .collect())
        },
        None => None,
    };
    let stdout = (&config["stdout"]).as_str();
    let stderr = (&config["stderr"]).as_str();

   //println!("stdout: {:#?}", stdout);


   let process = Process::new(String::from(cmd),
                                   working_dir,
                                   autostart,
                                   env,
                                   stdout,
                                   stderr
                                   );
    process.add_args()
        .add_workingdir()
        .add_env()
        .add_stdout()
        .add_stderr()
        .spawn();


    //let mut av: Vec<String> = Vec::new();
    //cmd.as_str().unwrap().split(' ').map(|little_str| av.push(String::from(little_str)));
    //let av: Vec<&str> = cmd.as_str().unwrap().split(' ').collect();


    /*let mut child = Command::new(av[0])
      .args(&av[1..])
      .env("PWD", working_dir.as_str().unwrap())
      .spawn()
      .unwrap();
      child.wait().unwrap();
      */
}

fn parse_config_file (filename: &str)
{
    let mut f = File::open(filename).expect("file not found");

    let mut contents = String::new();

    f.read_to_string(&mut contents)
        .expect("something went wrong reading the file");

    //println!("{}", contents);

    let docs = YamlLoader::load_from_str(&contents).unwrap();
    let doc = &docs[0];
    //println!("{:#?}", doc);

    assert!(!doc["programs"].is_badvalue());
    let x = &doc["programs"];
    {
        let hash = x.as_hash().unwrap();
        //   println!("hash: {:#?}", hash);
        for (name, cmd) in hash.iter() {
            exec_command(name, cmd);
        }
    }

    /*
       if let Some(h) = doc.as_hash() {
//println!("h[0] is {:#?}", h.0);
println!("{:?}", h[&Yaml::from_str("programs")]);
}
*/
/*
   let mut map = Yaml::Hash("lol");
   map.insert(Yaml::from_str("lala"), Yaml::from_str("lolo"));
//map.insert(2, "b");
let h = Yaml::Hash(map);
for (name, yaml) in h.iter().collect() {
println!("name is : {:?} yaml is : {:?} ", name, yaml);
}
*/
}

fn cli() {
    loop {
        print!("task_master> ");
        io::stdout().flush().unwrap();
        let mut guess = String::new();

        io::stdin().read_line(&mut guess)
            .expect("Failed to read line");

        println!("You wrote: {}", guess);
    }
}

fn main()
{
    let args: Vec<String> = env::args().collect();
    // println!("{:?}", args);
    let (option, filename) = parse_argv(&args);
    println!("{}, {}", option, filename);

    parse_config_file(filename);
    //    exec_command();
}
