mod api;

use crate::zinit;
use anyhow::{Context, Result};
use serde_yaml as encoder;
use std::path::{Path, PathBuf};

fn logger(level: log::LevelFilter) -> Result<()> {
    let logger = fern::Dispatch::new()
        .format(|out, message, record| {
            out.finish(format_args!(
                "zinit: {} ({}) {}",
                record.level(),
                record.target(),
                message
            ))
        })
        .level(level)
        .chain(std::io::stdout());
    let logger = match std::fs::OpenOptions::new().write(true).open("/dev/kmsg") {
        Ok(file) => logger.chain(file),
        Err(_err) => logger,
    };
    logger.apply()?;

    Ok(())
}

fn absolute<P: AsRef<Path>>(p: P) -> Result<PathBuf> {
    let p = p.as_ref();
    let result = if p.is_absolute() {
        p.to_path_buf()
    } else {
        let mut current = std::env::current_dir()?;
        current.push(p);
        current
    };

    Ok(result)
}

pub async fn init(cap: usize, config: &str, socket: &str, debug: bool) -> Result<()> {
    //std::fs::create_dir_all(config)?;
    if let Err(err) = logger(if debug {
        log::LevelFilter::Debug
    } else {
        log::LevelFilter::Info
    }) {
        eprintln!("failed to setup logging: {}", err);
    }

    let config = absolute(Path::new(config)).context("failed to get config dire absolute path")?;
    let socket = absolute(Path::new(socket)).context("failed to get socket file absolute path")?;

    debug!("switching to home dir: {}", config.display());
    std::env::set_current_dir(&config).with_context(|| {
        format!(
            "failed to switch working directory to '{}'",
            config.display()
        )
    })?;

    let init = zinit::ZInit::new(cap);

    init.serve();

    let services = zinit::config::load_dir(&config)?;
    for (k, v) in services {
        if let Err(err) = init.monitor(&k, v).await {
            error!("failed to monitor service {}: {}", k, err);
        };
    }

    let a = api::Api::new(init, socket);
    a.serve().await?;
    Ok(())
}

pub async fn list(socket: &str) -> Result<()> {
    let client = api::Client::new(socket);
    let results = client.list().await?;
    encoder::to_writer(std::io::stdout(), &results)?;
    Ok(())
}

pub async fn status(socket: &str, name: &str) -> Result<()> {
    let client = api::Client::new(socket);
    let results = client.status(name).await?;
    encoder::to_writer(std::io::stdout(), &results)?;
    Ok(())
}

pub async fn start(socket: &str, name: &str) -> Result<()> {
    let client = api::Client::new(socket);
    client.start(name).await?;
    Ok(())
}

pub async fn stop(socket: &str, name: &str) -> Result<()> {
    let client = api::Client::new(socket);
    client.stop(name).await?;
    Ok(())
}

pub async fn forget(socket: &str, name: &str) -> Result<()> {
    let client = api::Client::new(socket);
    client.forget(name).await?;
    Ok(())
}

pub async fn monitor(socket: &str, name: &str) -> Result<()> {
    let client = api::Client::new(socket);
    client.monitor(name).await?;
    Ok(())
}

pub async fn kill(socket: &str, name: &str, signal: &str) -> Result<()> {
    let client = api::Client::new(socket);
    client.kill(name, signal).await?;
    Ok(())
}
pub async fn logs(socket: &str) -> Result<()> {
    let client = api::Client::new(socket);
    client.logs(tokio::io::stdout()).await
}