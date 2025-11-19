#!/usr/bin/env bash

helloworld() {
    echo "Building Hello World example..."
    cargo zigbuild --release --example helloworld --target x86_64-unknown-linux-musl
    mkdir -p build/plugins/helloworld
    cp ../target/x86_64-unknown-linux-musl/release/examples/helloworld build/plugins/helloworld/helloworld
}

dynamic_capture_example() {
    echo "Building Dynamic Capture Example..."
    cargo zigbuild --release --example dynamic_capture_example --target x86_64-unknown-linux-musl
    mkdir -p build/plugins/dynamic_capture_example
    cp ../target/x86_64-unknown-linux-musl/release/examples/dynamic_capture_example build/plugins/dynamic_capture_example/dynamic_capture_example
}

static_capture_example() {
    echo "Building Static Capture Example..."
    cargo zigbuild --release --example static_capture_example --target x86_64-unknown-linux-musl
    mkdir -p build/plugins/static_capture_example
    cp ../target/x86_64-unknown-linux-musl/release/examples/static_capture_example build/plugins/static_capture_example/static_capture_example
}

api_call_example() {
    echo "Building API Call Example..."
    cargo zigbuild --release --example api_call_example --target x86_64-unknown-linux-musl
    mkdir -p build/plugins/api_call_example
    cp ../target/x86_64-unknown-linux-musl/release/examples/api_call_example build/plugins/api_call_example/api_call_example
}

event_subscriber_example() {
    echo "Building Event Subscriber Example..."
    cargo zigbuild --release --example event_subscriber_example --target x86_64-unknown-linux-musl
    mkdir -p build/plugins/event_subscriber_example
    cp ../target/x86_64-unknown-linux-musl/release/examples/event_subscriber_example build/plugins/event_subscriber_example/event_subscriber_example
}

helloworld
dynamic_capture_example
static_capture_example
api_call_example
event_subscriber_example

