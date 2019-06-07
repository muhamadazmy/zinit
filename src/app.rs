use crate::api;
use crate::manager;
use crate::settings;

use failure::Error;
use future::lazy;
use std::io::{self, BufRead};
use std::os::unix::net;
use std::path;
use tokio::prelude::*;

type Result<T> = std::result::Result<T, Error>;

/// init start init command, immediately monitor all services
/// that are defined under the config directory
pub fn init(config: &str) -> Result<()> {
    // load config
    std::env::set_current_dir(config)?;

    let configs = settings::load_dir(".", |file, err| {
        println!(
            "encountered err {} while loading file {:?}. skipping!",
            err, file
        );
        settings::Walk::Continue
    })?;

    // start the tokio runtime, start the process manager
    // and monitor all configured services
    // TODO:
    // We need to start the unix socket server that will
    // receive and handle user management commands (start, stop, status, etc...)
    tokio::run(lazy(|| {
        // creating a new instance from the process manager
        let manager = manager::Manager::new();

        // running the manager returns a handle that we can
        // use to actually control the process manager
        // currently the handle only exposes one method
        // `monitor` which spawns a new task on the process
        // manager given the configuration
        let handle = manager.run();

        for (name, config) in configs.into_iter() {
            if let Err(err) = handle.monitor(name, config) {
                error!("failed to monitor service: {}", err);
            }
        }

        if let Err(e) = api::run(handle) {
            error!("failed to start ctrl api {}", e);
        }

        Ok(())
    }));

    Ok(())
}

pub fn list() -> Result<()> {
    let p = path::Path::new("/var/run").join(api::SOCKET_NAME);
    let mut con = net::UnixStream::connect(p)?;
    con.write_all(b"list\n")?;

    let mut reader = io::BufReader::new(con);
    for line in reader.lines() {
        println!("line {:?}", line);
    }

    Ok(())
}
