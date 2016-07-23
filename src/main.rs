//! Implementation of a simple uTP client and server.
#[macro_use]
extern crate log;
extern crate env_logger;
extern crate utp;
extern crate getopts;
extern crate daemonize;

use daemonize::{Daemonize};
use std::process;
use std::process::Command;
use std::thread;
use std::net::{SocketAddr, SocketAddrV4, Ipv4Addr, UdpSocket};
use std::net;
use std::str;
use std::env;
use std::fs::File;
use std::path::Path;
use getopts::Options;

fn print_usage(program: &str, opts: Options) {
    let brief = format!("Usage: {} [options] REMOTE-LOCATION", program);
    print!("{}", opts.usage(&brief));
}

fn main() {
    use utp::{UtpStream, UtpListener, UtpSocket};
    use std::io::{stdin, stdout, stderr, Read, Write};

    // This example may run in either server or client mode.
    // Using an enum tends to make the code cleaner and easier to read.
    enum Mode {Server, Client}

    // Start logging
    env_logger::init().expect("Error starting logger");

    // Fetch arguments
    let args: Vec<String> = env::args().collect();
    let program = args[0].clone();
    let mut opts = Options::new();
    opts.optopt("o", "", "set output file name", "NAME");
    opts.optflag("s", "server", "server mode");
    opts.optflag("p", "port-range", "server listening port range");
    opts.optflag("h", "help", "print this help menu");

    let matches = match opts.parse(&args[1..]) {
        Ok(m) => { m }
        Err(f) => { panic!(f.to_string()) }
    };

    if matches.opt_present("h") {
        print_usage(&program, opts);
        return;
    }

    let input = if !matches.free.is_empty() {
        matches.free[0].clone()
    } else {
        print_usage(&program, opts);
        return;
    };

    // Parse the mode argument
    let mode: Mode = match matches.opt_present("s") {
        true => Mode::Server,
        false => Mode::Client
    };

    match mode {
        Mode::Server => {
            // Create a listening stream
            let addr = "0.0.0.0:55000";
            let mut stream = UdpSocket::bind(&addr).expect("Error binding stream");

            let mut f = File::open(input).unwrap();
            println!("{}", addr);

            let daemonize = Daemonize::new();

            let mut writer = stdout();
            let _ = writeln!(&mut stderr(), "Serving on {}", addr);

            // Create a reasonably sized buffer
            let mut payload = vec![0; 1024 * 1024];

            let mut startmagic = vec![0; 1];
            let (amount_read, sender_address) = stream.recv_from(&mut startmagic).unwrap();
            println!("got {} magic byte(s) from {:?}.", amount_read, sender_address);

            // match daemonize.start() {
            //     Ok(_) => { let _ = writeln!(&mut stderr(), "daemonized"); }
            //     Err(e) => { let _ = writeln!(&mut stderr(), "RWRWARWARARRR"); }
            // }

            let mut total = 0;
            // Create a reasonably sized buffer
            let mut payload = vec![0; 1300];
            loop {
                match stream.send_to(&payload, sender_address) {
                    Ok(written) => { print!("."); }
                    Err(e) => { panic!("{}", e); }
                }
            }
            // loop {
            //     match f.read(&mut payload) {
            //         Ok(read) => {
            //             println!("file read {}", read);
            //             match stream.write(&payload[0..read]) {
            //                 Ok(written) => {
            //                     total += written;
            //                     println!("written {}", total);
            //                 },
            //                 Err(e) => {
            //                     stream.close().expect("Error closing stream");
            //                     panic!("{}", e);
            //                 }
            //             }
            //         },
            //         Err(e) => {
            //             stream.close().expect("Error closing stream");
            //             panic!("{}", e);
            //         }
            //     }
            // }
        }
        Mode::Client => {
            let sections: Vec<&str> = input.split(":").collect();
            let addr: String = sections[0].to_owned();
            let path: String = sections[1].to_owned();
            let cmd = format!("~/bin/shoop -s {}", path);
            println!("addr: {}, path: {}, cmd: {}", addr, path, cmd);

            // let output = Command::new("ssh")
            //                      .arg(addr.to_owned())
            //                      .arg(cmd)
            //                      .output()
            //                      .unwrap_or_else(|e| {
            //                          panic!("failed to execute process: {}", e);
            //                      });
            // let udp_addr = String::from_utf8_lossy(&output.stdout).trim().to_owned();
            //
            // // Create a stream and try to connect to the remote address
            // println!("shoop server told us to connect to {}", udp_addr);
            let mut stream = UdpSocket::bind("0.0.0.0:0").expect("Error connecting to remote peer");
            println!("initted.");

            // let mut f = File::create("outfile").unwrap();
            // println!("created file.");
            // Create a reasonably sized buffer
            let mut payload = vec![0; 1024 * 1024];
            stream.connect("144.76.81.4:55000");
            stream.send(b"\x01");
            println!("write magic byte.");

            let mut total = 0;
            loop {
                match stream.recv(&mut payload) {
                    Ok(0) => {
                        println!("EOF");
                        break
                    },
                    Ok(read) => {
                        total += read;
                        println!("read {}", total);
                        // f.write_all(&payload[0..read-1]);
                    },
                    Err(e) => {
                        panic!("{:?}", e);
                    }
                }
            }

            // Explicitly close the stream.
        }
    }
}

