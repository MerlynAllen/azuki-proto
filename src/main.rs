use azukiproto::{Azuki, AzukiWorkMode, Config};
use log::{debug, error, info, trace, warn};
use log4rs::append::console::ConsoleAppender;
use log4rs::append::file::FileAppender;
use log4rs::config::Config as LogConfig;
use log4rs::config::{Appender, Logger, Root};
use log4rs::encode::pattern::PatternEncoder;
use std::io::{self, BufRead, Result, Write};
use std::net::{SocketAddr, ToSocketAddrs};
use std::thread::{self, JoinHandle};

fn run_server(config: Config) -> JoinHandle<()> {
    thread::Builder::new()
        .name("Server Main".to_string())
        .spawn(move || {
        let Config {
            bind_addr,
            bind_port,
            ..
        } = config;
        // config check
        let bind_addr = match bind_addr {
            Some(addr) => addr,
            None => {
                error!("bind_addr is not set");
                return;
            }
        };
        let bind_port = match bind_port {
            Some(port) => port,
            None => {
                error!("bind_port is not set");
                return;
            }
        };
        // bind
        let local_sock = SocketAddr::new(bind_addr, bind_port);
        debug!("Bind at {local_sock:?}");

        let mut azuki = Azuki::bind(bind_addr, bind_port);
        let mut azuki = match azuki {
            Err(e) => {
                panic!("Bind to {local_sock:?} failed! Reason: {e:?}");
            }
            Ok(res) => res,
        };
        azuki
            .listen(|peer, data, size| {
                info!(
                    "Received {size} bytes from {peer}\n{:?}",
                    String::from_utf8_lossy(data)
                );
            })
            .unwrap_or_else(|e| {
                panic!("Bind failed! Error {e:?}");
            });
        trace!("Binded.");
        azuki
            .thread_handler
            .expect("Unable to unpack thread_handler.")
            .join()
            .expect("Unable to join threads.");
    }).expect("Unable to spawn Server Main thread.")
}

fn run_client(config: Config) -> JoinHandle<()> {
    thread::Builder::new()
        .name("Client Main".to_string())
        .spawn(
        move || {
            let Config {
                bind_addr,
                bind_port,
                peer_addr,
                peer_port,
                ..
            } = config;
            // check config
            let bind_addr = match bind_addr {
                Some(addr) => addr,
                None => {
                    error!("bind_addr is not set");
                    return;
                }
            };
            let bind_port = match bind_port {
                Some(port) => port,
                None => {
                    error!("bind_port is not set");
                    return;
                }
            };
            let peer_addr = match peer_addr {
                Some(addr) => addr,
                None => {
                    error!("peer_addr is not set");
                    return;
                }
            };
            let peer_port = match peer_port {
                Some(port) => port,
                None => {
                    error!("peer_port is not set");
                    return;
                }
            };

            let local_sock = SocketAddr::new(bind_addr, bind_port);
            debug!("Bind at {local_sock:?}");
            let mut azuki = Azuki::bind(bind_addr, bind_port);
            let mut azuki = match azuki {
                Err(e) => {
                    panic!("Bind to {local_sock:?} failed! Reason: {e:?}");
                }
                Ok(res) => res,
            };
            azuki
                .connect(peer_addr, peer_port)
                .expect("Unable to connect to peer.");
            let message = "Hello, world!";
            let message = message.as_bytes().to_vec();
            azuki
                .send(&message)
                .expect("Unable to send data.");
        },
    ).expect("Unable to spawn Client Main thread.")
}

use clap::{Parser, Subcommand};
/// Azuki is a simple Transport-layer protocol.
#[derive(Parser, Debug)]
#[command(author, about, version, long_about = "Azuki is a simple Transport-layer protocol.")]
struct Args {
    /// Running mode (client/server/relay)
    #[clap(short, long, default_value = "client")]
    mode: AzukiWorkMode,

    /// The config file path, default use config.json
    #[clap(short, long, default_value = "config.json")]
    config: String,


    /// Set the log level. This option will override the log level in config file.
    /// Available values: trace, debug, info, warn, error, off
    #[clap(short, long, default_value = "info", verbatim_doc_comment)]
    log_level: String,
}

fn main() -> Result<()> {
    // Read cli args
    let args = Args::parse();

    // Init logger, use log4rs
    let log_pattern = "[{h({l})} {d(%Y-%m-%d %H:%M:%S %Z)(utc)}] [{T}:{I} {f}:{L}] {m}{n}";
    // read from config file
    let config_path = args.config;


    let config = Config::from_str(&config_path);
    let Config {
        bind_addr: _,
        bind_port: _,
        peer_addr: _,
        peer_port: _,
        mut log_level,
    } = config.clone();
    
    if args.log_level != "" { // overwrite with cli args
        log_level = Some(args.log_level);
    }
    if log_level.is_none() {
        log_level = Some("info".to_string());
    }

    let log_level = match log_level.unwrap().as_str() { // log_level will never be "None"
        "Trace" | "TRACE" | "trace" => log::LevelFilter::Trace,
        "Debug" | "DEBUG" | "debug" => log::LevelFilter::Debug,
        "Info"  | "INFO"  | "info"  => log::LevelFilter::Info,
        "Warn"  | "WARN"  | "warn"  => log::LevelFilter::Warn,
        "Error" | "ERROR" | "error" => log::LevelFilter::Error,
        _                           => log::LevelFilter::Info,
    };

    let log_out = ConsoleAppender::builder()
        .encoder(Box::new(PatternEncoder::new(log_pattern)))
        .build();
    let log_config = LogConfig::builder()
        .appender(Appender::builder().build("stdout", Box::new(log_out)))
        .build(Root::builder().appender("stdout").build(log_level))
        .unwrap();
    log4rs::init_config(log_config).unwrap();

    debug!("Set current log level from config: {}", log::max_level());

    let mode = args.mode;
    info!("Start with {mode:?} mode.");
    match mode {
        AzukiWorkMode::Client => {
            run_client(config.clone())
                .join()
                .expect(format!("{mode:?} worker thread join failed.").as_str());
        }
        AzukiWorkMode::Server => {
            run_server(config.clone())
                .join()
                .expect(format!("{mode:?} worker thread join failed.").as_str());
        }
        AzukiWorkMode::Relay => {
            error!("{mode:?} mode not implemented!");
            todo!()
        }
    }
    Ok(())
}
