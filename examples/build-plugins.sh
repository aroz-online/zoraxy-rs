#!/usr/bin/env bash

helloworld() {
    echo "Building Hello World example..."
    cargo build --release --example helloworld
    mkdir -p build/plugins/helloworld
    cp ../target/release/examples/helloworld build/plugins/helloworld/helloworld
}

dynamic_capture_example() {
    echo "Building Dynamic Capture Example..."
    cargo build --release --example dynamic_capture_example
    mkdir -p build/plugins/dynamic_capture_example
    cp ../target/release/examples/dynamic_capture_example build/plugins/dynamic_capture_example/dynamic_capture_example
}

static_capture_example() {
    echo "Building Static Capture Example..."
    cargo build --release --example static_capture_example
    mkdir -p build/plugins/static_capture_example
    cp ../target/release/examples/static_capture_example build/plugins/static_capture_example/static_capture_example
}

helloworld
dynamic_capture_example
static_capture_example

