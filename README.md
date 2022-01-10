# ddns-cloudflare

[![](https://img.shields.io/github/workflow/status/pkoenig10/ddns-cloudflare/CI?label=ci)][actions]
[![](https://img.shields.io/github/workflow/status/pkoenig10/ddns-cloudflare/Release?label=release)][actions]

Custom Dynamic DNS program to update [Cloudflare](https://www.cloudflare.com/dns/) DNS records with current external IP address.

## How it works

Program sends [special DNS query to Cloudflare](https://community.cloudflare.com/t/can-1-1-1-1-be-used-to-find-out-ones-public-ip-address/14971) public DNS server in order to obtain computer's external IP address. Saves this address and updates specified Cloudflare DNS address if needed.

## Installation

You need to install Rust in order to build this program. Please check this [link](https://rustup.rs/).
Just run `cargo build` to build a binary that can be found in `target/debug/ddns-cloudflare`.

To cross-compile for aarch64 (Raspberry PI):
1. Add aarch64 target: `rustup target add aarch64-unknown-linux-gnu`
1. Install aarch64 compiler and linker: `sudo apt install gcc-aarch64-linux-gnu`
1. Update `~/.cargo/config` with following lines:
    ```
    [target.aarch64-unknown-linux-gnu]
    linker = "aarch64-linux-gnu-gcc"
    ```
1. Build: `cargo build --target aarch64-unknown-linux-gnu --release`


## Usage

[Create Cloudlare API token](https://developers.cloudflare.com/api/tokens/create) with zone edit permissions.

Export following environment variables:
- `API_TOKEN` - Cloudflare API token
- `DOMAIN` - subdomain that will be used to resolve external IP address, ie `myip.example.com`, where `example.com` is DNS zone managed by Cloudflare

By default program caches ipv4 address in file `/tmp/cloudflare-ddns.txt`, if cached address matches current ipv4 external address then no action is performed, otherwise program creates/updates specified `DOMAIN` DNS records with both ipv4 and ipv6 addresses.


[actions]: https://github.com/pkoenig10/ddns-cloudflare/actions
