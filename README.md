# Brainf Interpreter in Rust

This is a Rust-based interpreter for the Brainf programming language.
The interpreter supports reading Brainf programs from files and taking input directly from the command line.

## Requirements

- Rust (with Cargo) installed on your system. You can get it from [here](https://www.rust-lang.org/tools/install).

## Installation

To build the project, run the following command in the root directory of the project:

```sh
cargo build --release

```
This will create an executable in the target/release directory.

## Usage
To run the interpreter, use the following command:

```sh
cargo run --release -- -f <file_path> -i
```

## Contributing
Contributions are welcome! Please feel free to submit a pull request or open an issue.

## License
This project is licensed under the MIT License.
