A script to download random git.io shortlinks.

Across machines:

```
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
cargo run --release | pv --line-mode --rate > links.txt
```

If the numbers seem low, try tweaking `NUM_WORKERS` in the code.
