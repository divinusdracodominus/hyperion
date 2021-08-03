use scrypt::password_hash::{rand_core::OsRng, SaltString};
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;

fn main() {
    let salt = SaltString::generate(&mut OsRng);
    let path = PathBuf::new().join("src").join("salt.txt");
    if !path.exists() {
        let mut file = File::create(path).unwrap();
        file.write_all(salt.as_bytes()).unwrap();
    }
}
