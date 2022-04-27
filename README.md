A script to download git.io shortlinks.

Across machines:

## Random mode

Download random URLs

```
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
cargo run --release random | pv --line-mode --rate > links.txt
```

## Stdin mode

Download URLs from stdin

```
...
echo https://git.io/foobar > input.txt
cargo run --release stdin < input.txt | pv --line-mode --rate > links.txt
```

If the numbers seem low, try tweaking `NUM_WORKERS` in the code.
