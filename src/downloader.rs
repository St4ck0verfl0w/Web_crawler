use crossbeam_channel::Receiver;
use crossbeam_channel::Sender;
use dashmap::DashMap;
use tokio::runtime::Runtime;
use reqwest::Client;
use select::document::Document;
use select::predicate::Name;
use std::fs;
use std::path::Path;
use std::sync::Arc;
use std::{thread, time};
use url::Url;

async fn fetch(url: &String) -> Result<String, reqwest::Error> {
    println!("Downloading {}", url);
    match Client::new().get(url).send().await {
        Ok(r) => r.text().await,
        Err(e) => Err(e),
    }
}

fn save_to_file(root: &String, url: Url, text: &String) {
    let absolute_path = format!("{}{}", root, url.path());
    let path = Path::new(&absolute_path);
    if !path.parent().is_none() {
        fs::create_dir_all(path.parent().unwrap()).unwrap();
    }
    match fs::write(absolute_path, text.to_string()) {
        Ok(_) => (),
        Err(r) => println!(
            "Error while writing content from url {} to file {}:\n{}",
            url.as_str(),
            url.path(),
            r
        ),
    };
}

fn process_all_links(s: &Sender<String>, used_urls: &DashMap<String, bool>, html_output: &String) {
    Document::from(html_output.as_str())
        .find(Name("a"))
        .filter_map(|n| n.attr("href"))
        .for_each(|x| {
            if !used_urls.contains_key(x) {
                used_urls.insert(x.to_string(), true);
                match s.send(x.to_string()){
                    Ok(_) => (),
                    Err(e) => println!("error while sending url to channel:\n{}",e),
                };
            }
        });
}

pub fn download_all(
    s: Sender<String>,
    r: Receiver<String>,
    used_urls: Arc<DashMap<String, bool>>,
    root: Arc<String>,
) -> () {

    println!("New thread started");


    loop {
        if !r.is_empty() {
            match r.recv() {
                Ok(url) => match Url::parse(&url) {
                    Ok(parsed) => match Runtime::new().expect("Failed to created Tokio runtime").block_on(fetch(&url)) {
                        Ok(html_output) => {
                            save_to_file(&root, parsed, &html_output);
                            process_all_links(&s, &used_urls, &html_output);
                        }
                        Err(e) => println!("{}", e), // could not fetch
                    },
                    Err(_) => (/*could not be parsed as url*/),
                },
                Err(e) => println!("error while receiving from share channel:\n{}", e),
            }
        } else {
            thread::sleep(time::Duration::from_millis(10));
        }
    }
}
