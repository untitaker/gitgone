use rand::{SeedableRng, Rng};

#[tokio::main]
async fn main() {
    let client = reqwest::ClientBuilder::new().redirect(reqwest::redirect::Policy::none()).build().unwrap();

    const NUM_WORKERS: usize = 200;
    let mut workers = Vec::new();

    for _ in 0..NUM_WORKERS {
        let client = client.clone();
        workers.push(tokio::spawn(async move {
            let mut rng = rand::rngs::SmallRng::from_entropy();

            loop {
                const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";
                let mut line = "https://git.io/".to_owned();

                for _ in 0..5 {
                    line.push(CHARSET[rng.gen_range(0..CHARSET.len())] as char);
                }

                let res: reqwest::Response = match client.get(&line)
                    .send()
                    .await {
                        Ok(x) => x,
                        Err(e) => {
                            eprintln!("failed to send: {}", e);
                            continue;
                        }
                    };
                if res.status().as_u16() == 404 {
                    println!("{} -> 404", line);
                } else if let Some(location) = res.headers().get("Location") {
                    println!("{} -> {}", line, location.to_str().unwrap());
                }
                res.bytes().await.ok();
            }
        }));
    }

    for worker in workers {
        worker.await.unwrap();
    }
}
