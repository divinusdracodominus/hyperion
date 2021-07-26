## Purpose
This is designed as a replacement to the LAMP stack (Linux Apache MySQL and PHP) specifically its goal is to make an easy to use, clean, strongly typed PHP alternative, with a simple web server backend.

## How This Tool is Built
This program, and stack relies on ion shell, which is a bash like shell written in rust [see here](https://doc.redox-os.org/ion-manual/html/)

### How hyperion replaces LAMP
The Linux Component isn't replaced, rather ion runs just the same on linux, mac, and redox os, however apache is replaced with a minimal async [TcpStream](https://docs.rs/tokio/1.9.0/tokio/net/struct.TcpStream.html) [http parser](https://crates.io/crates/http) and ion (perhaps in the future toml) to replace http access (.htaccess). 

MySQL is for the moment replaced with the [sqlite crate](https://crates.io/crates/sqlite) for SQL in a portable manner. This tool focuses on efficiency, security, and protability. (for now at least)

## Future Plans / Ideas
A module system based on dynamically linking [wasmtime](https://crates.io/crates/wasmtime) wasm modules.

### Why Use WasmTime
The idea behind using wasmtime is to ensure a minimal layer of security for potentially hazardous server side ion builtins, this also means ion only needs to be ported to one platform (instead of windows, BSD, android etc)
