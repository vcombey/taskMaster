# TaskMaster

A 42 school project. The goal of this project is to make a job control daemon, with features
similar to supervisor.

## Getting Started

We have chosen a Client-server architecture. And the server manages programs in a multithread fashion.

### Installing


```
Cargo run --bin server -- -c [config_file]
Cargo run --bin client
```

Or

```
Cargo build
./target/debug/server -c [config_file]
./target/debug/client
```

The config file must be in YAML. An exemple of configuration file is present in the ressources directory.

## Running the tests

Explain how to run the automated tests for this system

### Tests

Explain what these tests test and why

```
Give an example
```


## Built With

* [Serde](https://crates.io/crates/serde) - Serialiser Deserialiser
* [Maven](https://crates.io/crates/yaml-rust/) - Yaml parser in rust


## Authors
* **DE SEDE Adrien** - [Ixskill](https://github.com/Ixskill)
* **COMBEY Vincent** - [vcombey](https://github.com/vcombey)
