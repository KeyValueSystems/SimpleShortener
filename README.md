# Simple URL shortener

A very simple URL shortener, which is easy to configure and quite speedy.
Later it is planned to add some analytics.

If you have any issues you can contact me on discord, `valkyrie_pilot#2707`, or via email [valk@randomairborne.dev](valk@randomairborne.dev)

You can edit links at /simpleshortener/ on the domain you use to host it.

## Install
`docker pull ghcr.io/randomairborne/simpleshortener`
### Unstable
`docker pull ghcr.io/randomairborne/simpleshortener:master`


## Building
You can build from source with [rust](https://rust-lang.org)
```bash
git clone https://github.com/randomairborne/SimpleShortener.git
cargo build --release
```