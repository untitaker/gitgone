use clap::{Parser, Subcommand};
use rand::{Rng, SeedableRng};
use std::sync::Arc;
use tokio::io;
use tokio::io::AsyncBufReadExt;

/// Simple program to greet a person
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// Name of the person to greet
    #[clap(subcommand)]
    mode: Mode,
}

#[derive(Subcommand, Debug)]
enum Mode {
    Random,
    Stdin,
}

async fn download(client: &reqwest::Client, line: &str) {
    let res: reqwest::Response = match client.get(line).send().await {
        Ok(x) => x,
        Err(e) => {
            eprintln!("failed to send: {}", e);
            return;
        }
    };
    if res.status().as_u16() == 404 {
        println!("{} -> 404", line);
    } else if let Some(location) = res.headers().get("Location") {
        println!("{} -> {}", line, location.to_str().unwrap());
    } else {
        eprintln!("odd response: {}", res.status().as_u16());
    }
    res.bytes().await.ok();
}

#[tokio::main]
async fn main() {
    let args = Args::parse();
    let client = reqwest::ClientBuilder::new()
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .unwrap();

    const NUM_WORKERS: usize = 200;
    const QUEUE_BUF: usize = 5000;
    let mut workers = Vec::new();

    match args.mode {
        Mode::Random => {
            for _ in 0..NUM_WORKERS {
                let client = client.clone();
                workers.push(tokio::spawn(async move {
                    let mut rng = rand::rngs::SmallRng::from_entropy();

                    loop {
                        const CHARSET: &[u8] =
                            b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789_-";
                        let mut line = "https://git.io/".to_owned();

                        for _ in 0..5 {
                            line.push(CHARSET[rng.gen_range(0..CHARSET.len())] as char);
                        }

                        download(&client, &line).await;
                    }
                }));
            }
        }
        Mode::Stdin => {
            let queue = Arc::new(deadqueue::limited::Queue::new(QUEUE_BUF));
            for _ in 0..NUM_WORKERS {
                let client = client.clone();
                let receiver = queue.clone();
                workers.push(tokio::spawn(async move {
                    while let Some(line) = receiver.pop().await {
                        let line = line.replace("http://git.io/", "https://git.io/");

                        download(&client, &line).await;
                    }
                }));
            }

            let mut lines = io::BufReader::new(io::stdin()).lines();
            while let Some(line) = lines.next_line().await.unwrap() {
                let line = line.trim();
                queue.push(Some(line.to_owned())).await;
            }

            for _ in 0..NUM_WORKERS {
                queue.push(None).await;
            }
        }
    }

    for worker in workers {
        worker.await.unwrap();
    }
}
