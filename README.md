# TaskMaster

A 42 school project. The goal of this project is to make a job control daemon, with features
similar to [supervisor](http://supervisord.org/).

Subject can be found in the ressource directory !

## Getting Started

We have chosen a Client-server architecture, with the server managing programs in a multithreaded fashion.

### Installing


```
cargo run --bin server -- -c [config_file]
cargo run --bin client
```

Or

```
cargo build
./target/debug/server -c [config_file]
./target/debug/client
```

The config file must be in YAML. Exemples of configuration file with the available option are present in the ressources directory.
The option's behavior should be very similar to supervisor's behavior with equivalent options.

## Tests

Automated test concern mainly the parsing of the command on the client side and the parsing of the config file on the server side.

### Running the tests

```
cargo test
```


## Built With

* [Serde](https://crates.io/crates/serde) - Serialiser Deserialiser
* [Maven](https://crates.io/crates/yaml-rust/) - Yaml parser in rust


## Authors
* **DE SEDE Adrien** - [ade-sede](https://github.com/ade-sede)
* **COMBEY Vincent** - [vcombey](https://github.com/vcombey)
