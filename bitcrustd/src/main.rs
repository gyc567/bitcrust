#[macro_use]
extern crate clap;
#[macro_use]
extern crate log;
extern crate bitcrust_net;
extern crate simple_logger;
extern crate multiqueue;
extern crate rusqlite;

use std::thread;
use std::time::Duration;

use bitcrust_net::{BitcoinNetworkConnection, BitcoinNetworkError, Message};

use clap::{App, Arg, ArgMatches, SubCommand};
use log::LogLevel;

mod client_message;
mod peer_manager;
mod peer;

use peer_manager::PeerManager;

fn main() {
    let matches = App::new("bitcrustd")
        .version(crate_version!())
        .author("Chris M., Tomas W.")
        .arg(Arg::with_name("debug")
            .short("d")
            .multiple(true)
            .help("Turn debugging information on"))
        .subcommand(SubCommand::with_name("node").about("Bitcrust peer node"))
        .subcommand(SubCommand::with_name("stats")
            .about("Get stats from a running Bitcrust node")
            .arg(Arg::with_name("host")
                .short("h")
                .takes_value(true))
            .subcommand(SubCommand::with_name("peers"))
        )
        .subcommand(SubCommand::with_name("balance")
            .about("Get balance for address")
            .arg(Arg::with_name("address")
                .short("a")
                .help("Address to get balance for")
                .takes_value(true)
                .required(true))
        )
        .get_matches();

    let log_level = match matches.occurrences_of("debug") {
        0 => LogLevel::Warn,
        1 => LogLevel::Info,
        2 => LogLevel::Debug,
        3 | _ => LogLevel::Trace,
    };
    

    match matches.subcommand() {
        ("node", Some(node_matches)) => {
            simple_logger::init_with_level(log_level).expect("Couldn't initialize logger");
            node(node_matches);
        }
        ("balance", Some(balance_matches)) => {
            balance(balance_matches);
        }
        ("stats", Some(stats_matches)) => {
            stats(stats_matches);
        }
        ("", None) => println!("No subcommand was used"), // If no subcommand was usd it'll match the tuple ("", None)
        _ => unreachable!(), // If all subcommands are defined above, anything else is unreachabe!()
    }
}

fn node(_matches: &ArgMatches) {
    let mut client = PeerManager::new();
    client.execute();
}

fn balance(matches: &ArgMatches) {
    // This unwrap is safe because we require it above
    let address = matches.value_of("address").unwrap();
    println!("I'd love to get your balance on '{}' but it's not yet implemented!", address);
}

fn stats(matches: &ArgMatches) {
    let host = matches.value_of("host").unwrap_or("127.0.0.1:8333").to_string();
    match matches.subcommand() {
        ("peers", Some(peer_matches)) => {
            connected_peers(peer_matches, host);
        }
        ("", None) => println!("No subcommand was used"), // If no subcommand was usd it'll match the tuple ("", None)
        _ => unreachable!(), // If all subcommands are defined above, anything else is unreachabe!()
    }
}

fn connected_peers(matches: &ArgMatches, host: String) {
    let connection = BitcoinNetworkConnection::new(host.clone())
        .expect(&format!("Couldn't connect to a node running on {}", host));
    let _ = connection.try_send(peer::Peer::version());
    loop {
        if let Some(msg) = connection.try_recv() {
            match msg {
                Ok(msg) => match msg {
                    Message::Version(version) => {
                        let _ = connection.try_send(Message::Verack);
                        break;
                    }
                    _ => {}
                },
                Err(BitcoinNetworkError::ReadTimeout) => thread::sleep(Duration::from_millis(200)),
                Err(BitcoinNetworkError::Closed) => {
                    println!("Remote server closed the connection");
                    return
                }
                Err(BitcoinNetworkError::BadBytes) => return
            }
        }
    }
    match connection.try_send(Message::BitcrustPeerCountRequest) {
        Ok(_) => {},
        Err(e) => warn!("Error sending request: {:?}", e),
    }
    loop {
        if let Some(msg) = connection.try_recv() {
            match msg {
                Ok(msg) => match msg {
                    Message::BitcrustPeerCount(count) => {
                        println!("There are {} peers currently connected to {}", count, host);
                        break;
                    }
                    _ => {}
                },
                Err(BitcoinNetworkError::ReadTimeout) => thread::sleep(Duration::from_millis(200)),
                Err(BitcoinNetworkError::Closed) => {
                    println!("Remote server closed the connection");
                    return
                }
                Err(BitcoinNetworkError::BadBytes) => return
            }
        }
    }
}