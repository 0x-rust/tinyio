use clap::Parser;
use hdrhistogram::{Histogram, RecordError};
use rand::Rng;
use std::io::{Read, Write};
use std::net::TcpStream;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{mpsc, Arc};
use std::thread;
use std::time::{Duration, Instant};

const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ\
                            abcdefghijklmnopqrstuvwxyz\
                            0123456789)(*&^%$#@!~";
struct Stats {
    inb: u64,
    outb: u64,
    hist: Histogram<u64>,
}

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct RunArgs {
    nthreads: u8,
    msg_size: usize,
    host_port: String,
}

fn main() {
    let args = RunArgs::parse();
    let run_time_secs = 10;
    let nthreads = args.nthreads;
    let run_duration = Duration::from_secs(run_time_secs);
    let test_start = Instant::now();

    let msg_size: usize = args.msg_size;

    let (send, receive) = mpsc::channel();
    let stop = Arc::new(AtomicBool::new(false));
    for _ in 0..nthreads {
        let stop = stop.clone();
        let stats_send = send.clone();
        let address = args.host_port.clone();
        thread::spawn(move || {
            let mut stream = TcpStream::connect(&address).unwrap();
            let data = rand_bytes(msg_size);
            let mut in_buf: Vec<u8> = vec![0; msg_size];
            let mut stats = Stats {
                inb: 0,
                outb: 0,
                hist: Histogram::new_with_max(30 * 1000_000, 2).unwrap(),
            };

            while !stop.load(Ordering::Relaxed) {
                let start = Instant::now();

                match stream.write_all(&data) {
                    Err(_) => {
                        println!("Write error!");
                        break;
                    }
                    Ok(_) => stats.outb += 1,
                }

                match stream.read(&mut in_buf) {
                    Err(_) => break,
                    Ok(m) => {
                        if m == 0 || m != msg_size {
                            println!("Read error! length={}", m);
                            break;
                        }
                    }
                };
                stats.inb += 1;
                let duration = start.elapsed();
                let latency_micros = duration.as_micros() as u64;
                if stats.hist.record(latency_micros).is_err() {
                    println!("record error ");
                    break;
                }
            }
            stats_send.send(stats);
        });
    }

    thread::sleep(Duration::from_secs(run_time_secs));
    (*stop).store(true, Ordering::Relaxed);

    let mut all_stats = Stats {
        inb: 0,
        outb: 0,
        hist: Histogram::new_with_max(30 * 1000_000, 2).unwrap(),
    };
    for _ in 0..nthreads {
        let thread_stats = receive.recv().unwrap();
        all_stats.inb += thread_stats.inb;
        all_stats.outb += thread_stats.inb;
        all_stats.hist.add(thread_stats.hist);
    }

    print_stats(run_time_secs, &all_stats);
}

fn print_stats(run_time_secs: u64, stats: &Stats) {
    println!(
        "Throughput: {} request/sec, {} response/sec",
        stats.outb / run_time_secs,
        stats.inb / run_time_secs
    );
    println!("Requests: {}", stats.outb);
    println!("Responses: {}", stats.inb);
    let hist = &stats.hist;
    println!(
        "latency in micros min={}, mean={}, 50th={}, 90th={}, 99.99={}, max={}",
        hist.min(),
        hist.mean(),
        hist.value_at_quantile(0.5),
        hist.value_at_quantile(0.9),
        hist.value_at_quantile(0.99),
        hist.max()
    );
}

fn rand_bytes(n: usize) -> Vec<u8> {
    let mut rng = rand::thread_rng();

    let bytes_rand: Vec<u8> = (0..n)
        .map(|_| {
            return rng.gen_range(0..256) as u8;
        })
        .collect();
    return bytes_rand;
}
