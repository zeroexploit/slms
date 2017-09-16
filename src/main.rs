#[macro_use]
extern crate lazy_static;
extern crate daemonize;

mod media;
mod tools;
mod database;
mod configuration;
mod upnp;
mod server;
mod provider;

use std::env;
use server::MediaServer;

fn main() {
    // Set some defaults
    let mut helpscreen: bool = false;
    let mut daemonize: bool = true;
    let mut cfg_path: String = String::from("/etc/slms/server.cfg");
    let args: Vec<String> = env::args().collect();

    // Check for Arguments that override the defaults
    for index in 0..args.len() {
        if args[index] == "-c" || args[index] == "--configuration" {
            if index + 1 < args.len() {
                cfg_path = args[index + 1].clone();
            }
        } else if args[index] == "-d" || args[index] == "--dont-daemonize" {
            daemonize = false;
        } else if args[index] == "-h" || args[index] == "--help" {
            helpscreen = true;
        }
    }

    // Run the Media Server
    MediaServer::run(&cfg_path, daemonize, helpscreen);
}
