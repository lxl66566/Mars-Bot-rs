# Mars-Bot-rs

A toy telegram mars bot. "Mars" means a message (here only image) appears once again. So this bot can mention you when an image appears twice or more.

Support running on linux and windows.

## Usage

1. Install Mars-Bot-rs:
   - Download the archive in [Release](https://github.com/lxl66566/Mars-Bot-rs/releases)
2. extract and run: `./mars-bot -t <TOKEN>`
3. add this bot to telegram public group you want to use.

advanced:

1. run in linux server background, use `systemd-run ./mars-bot -t <TOKEN>`.
2. `./mars-bot -h` to see all commands.
3. `./mars-bot e` to export a default config file. You can edit the config file and restart it.
4. `RUST_LOG=trace ./mars-bot` to see more detailed output.
5. you can also set token in config file: `token = xxx` or in env: `export TELOXIDE_PROXY=xxx`
6. default storage position (db + config): `~/.local/mars-bot`

## Features

There are 2 backend that can be used in Mars-Bot-rs:

- Sled (Default)
- Sqlite

If you want to use other backends, you need to compile Mars-Bot-rs manually.

```sh
git clone https://github.com/lxl66566/Mars-Bot-rs.git
cargo +nightly install --path Mars-Bot-rs --features sqlite
```
