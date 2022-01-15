use clap::Parser;
use reqwest::Client;

/// A simple tool to web crawl a part of a website
#[derive(Parser,Debug)]
#[clap(author, version, about, long_about = None)]
struct Args{

    /// URL of the first target
    #[clap(short,long)]
    url: String,

    /// Path to the directory where the content will be downloaded
    #[clap(short, long, default_value = "./")]
    dir: String,
}

async fn download(url: String) -> Result<String,reqwest::Error>{
    println!("downloading {}", url);
    match Client::new().get(&url).send().await{
        Ok(r) => {
            r.text().await
        },
        Err(e) => Err(e)
    }
}

#[tokio::main]
async fn main() {

    let args = Args::parse();
    println!("Downloading {} to local dir {}",args.url, args.dir);
    match download(args.url).await{
        Ok(r) => println!("{}",r),
        Err(m) => println!("{}",m),
    }
}
