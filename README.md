![crates.io](https://img.shields.io/crates/v/zoraxy-rs)

# Zoraxy-rs

Zoraxy-rs is a Rust crate for building plugins for [Zoraxy](https://github.com/tobychui/zoraxy)

The examples have been verified to work with Zoraxy v3.2.9

## Oddities

### If using docker, and you see something like `[plugin-manager] [system:error] Failed to load plugin: ...: exit status 127` in the logs

Open a shell in the container, and run `ldd` on the plugin binary, e.g.

```sh
docker exec -it <container_id> /bin/sh -c 'ldd /opt/zoraxy/plugin/api_call_example/api_call_example'
```

you might see that some libraries are missing, this is because the plugin was built against `glibc` but the docker container uses `musl`.

Fixing this could be as easy as changing some feature flags to ensure openssl isn't being used, but could be more involved requiring you to build the plugin with the `x86_64-unknown-linux-musl` target, which is easiest to do using `cargo-zigbuild`.

```sh
rustup target add x86_64-unknown-linux-musl
cargo install cargo-zigbuild
cargo zigbuild --release --example api_call_example --target x86_64-unknown-linux-musl
```

You might need to do some additional setup to get the `musl` target working, see [the Rust documentation](https://doc.rust-lang.org/stable/rustc/platform-support.html#musl-targets) for more details.