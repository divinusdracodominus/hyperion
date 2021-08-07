use crate::config::Config;
use rayon::prelude::*;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::thread;
use std::thread::JoinHandle;
use ion_shell::Shell;
/// a struct for handling web socket requests
#[derive(Debug, Clone)]
pub struct SocketHandler {
    config: Config,
    sessions: Arc<RwLock<HashMap<String, HashMap<String, String>>>>,
}

impl SocketHandler {
    pub fn new(
        config: Config,
        sessions: Arc<RwLock<HashMap<String, HashMap<String, String>>>>,
    ) -> Self {
        Self { config, sessions }
    }

    pub fn launch(self) -> Vec<JoinHandle<Result<(), ws::Error>>> {
        if let Some(sockets) = self.config.sockets {
            sockets
                .into_par_iter()
                .map(|socket| {
                    let addr = socket;
                    thread::spawn(move || {
                        ws::listen(addr, move |out| { 
                            let mut Shell = Shell::new();    
                            move |_msg| Ok(()) 
                        })
                    })
                })
                .collect::<Vec<JoinHandle<Result<(), ws::Error>>>>()
        } else {
            Vec::new()
        }
    }
}
