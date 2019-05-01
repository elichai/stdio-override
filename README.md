# stdio-override
[![Build Status](https://travis-ci.com/elichai/stdio-override.svg?branch=master)](https://travis-ci.com/elichai/stdio-override)
[![Latest version](https://img.shields.io/crates/v/stdio-override.svg)](https://crates.io/crates/stdio-override)
[![Documentation](https://docs.rs/stdio-override/badge.svg)](https://docs.rs/stdio-override)
![License](https://img.shields.io/crates/l/stdio-override.svg)

A Rust library to easily override Stdio file descriptors in Rust

* [Documentation](https://docs.rs/stdio-override)

## Usage

Add this to your `Cargo.toml`:

```toml
[dependencies]
stdio-override = "0.1"

```

and for Rust Edition 2015 add this to your crate root:

```rust
extern crate stdio_override;
```
In Rust Edition 2018 you can simply do:
```rust
use stdio_override::*;
```

Here's an example on how to write stdout into a file:

```rust
    use std::{fs::read_to_string, mem};
    use stdio_override::StdoutOverride;
    
    fn main() {
        let file_name = "./test.txt";
    
        let guard = StdoutOverride::override_file(file_name).unwrap();
        println!("12345");
        mem::drop(guard);
    
        let contents = read_to_string(file_name).unwrap();
        assert_eq!("12345\n", contents);
        println!("Outside!");
    }
```

You can do the same with sockets:
```rust
    use std::{
        io::Read,
        mem,
        net::{TcpListener, TcpStream},
    };
    use stdio_override::StdoutOverride;
    
    fn main() {
        let address = ("127.0.0.1", 5543);
    
        let listener = TcpListener::bind(address).unwrap();
        let socket = TcpStream::connect(address).unwrap();
    
        let guard = StdoutOverride::override_raw(socket).unwrap();
        println!("12345");
        mem::drop(guard);
    
        let mut contents = String::new();
        let (mut stream, _) = listener.accept().unwrap();
        stream.read_to_string(&mut contents).unwrap();
    
        assert_eq!("12345\n", contents);
    
        println!("Outside!");
    }
```