use clap::Parser;

use crate::downloader::download_all;
use crossbeam_channel::unbounded;
use dashmap::DashMap;
use std::sync::Arc;
use std::thread;

mod downloader;

/// A simple tool to web crawl a part of a website
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// URL of the first target
    #[clap(short, long)]
    url: String,

    /// Path to the directory where the content will be downloaded
    #[clap(short, long, default_value = "./")]
    dir: String,

    /// Number of threads downloading
    #[clap(short, long, default_value_t = 1)]
    threads: u32,
    //     /// Max number of file downloading (-1 if infinite)
    //     #[clap(short, long, default_value_t = -1)]
    //     files: i32,
}

#[tokio::main]
async fn main() {
    let args = Args::parse();
    println!(
        "Crawling all urls from {} to local dir {}",
        args.url, args.dir
    );

    let (s, r) = unbounded();
    let used_urls: DashMap<String, bool> = DashMap::new();
    let shared_used_urls = Arc::new(used_urls);
    let shared_dir = Arc::new(args.dir);

    shared_used_urls.insert(args.url.clone(), true);
    s.send(args.url.clone()).unwrap();
    let mut threads = Vec::with_capacity(args.threads as usize);
    for _ in 0..args.threads {
        let (si, ri) = (s.clone(), r.clone());
        let shared_dir = Arc::clone(&shared_dir);
        let shared_used_urls = Arc::clone(&shared_used_urls);
        threads.push(thread::spawn(move || {
            download_all(si, ri, shared_used_urls, shared_dir)
        }));
    }

    for thread in threads{
        thread.join().unwrap();
    }

    println!("DONE");
}
