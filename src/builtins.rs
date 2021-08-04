use ion_shell::{builtins::Status, types, types::Str, Shell};
use small::string::String as string;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

/// this type alias serves as a means of documenting "@SESSION" variable set in ion
/// on each request "@SESSION" is loaded from an Arc<RwLock<HashMap<String, HashMap<String, String>>>>
/// where the first HashMap maps SESSIONID to SESSION variables
/// ## Set and Read Session
/// ```ignore
/// session_start
/// set_session_variable "active" "true"
/// let active = @SESSION["active"]
/// ```
pub type SESSION = HashMap<String, String>;

use scrypt::{
    password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Scrypt,
};

const SALTSTRING: &str = include_str!("salt.txt");

/// use script to hash a given string
pub fn scrypt_hash(args: &[types::Str], _shell: &mut Shell) -> Status {
    let salt = SaltString::new(SALTSTRING).unwrap();
    // Hash password to PHC string ($scrypt$...)
    let password_hash = Scrypt
        .hash_password_simple(args[1].as_bytes(), salt.as_ref())
        .unwrap()
        .to_string();
    println!("{}", password_hash);
    Status::SUCCESS
}

/// used to verify passwords generated with scrypt
///
/// # Arguments
/// the normal arguments found here are components of ion_shell builtin functions
/// the form that the script function takes on is
/// scrypt_verify "password" "hash"
pub fn scrypt_verify(args: &[types::Str], _shell: &mut Shell) -> Status {
    let parsed_hash = match PasswordHash::new(args[2].as_str()) {
        Ok(v) => v,
        Err(e) => { return Status::error(format!("{}", e)); }
    };
    match Scrypt.verify_password(args[1].as_bytes(), &parsed_hash) {
        Ok(_) => println!("true"),
        Err(e) => {
            println!("{}", e);
            //return Status::error(format!("{}", e));
        }
    }

    Status::SUCCESS
}

/*pub fn open_sqlite_conn(args: &[Str], _shell: &mut Shell) -> Status {
    if(args.len() != 2) {
        return Status::error("expected only one argument: dbname");
    }
    let connections = DBCONNS.write().unwrap();
    if let Some(conn) = connections.get(args[1]) {
        Status::SUCCESS
    }else{
        match sqlite::Connection::open(args[1]) {
            Ok(conn) => {
                connections.insert(args[1].clone(), conn);
                Status::SUCCESS
            },
            Err(e) => Status::error("unable to open database connection")
        }
    }
}*/
