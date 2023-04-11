# enum2contract

[<img alt="github" src="https://img.shields.io/badge/github-matthewjberger/enum2contract-8da0cb?style=for-the-badge&labelColor=555555&logo=github" height="20">](https://github.com/matthewjberger/enum2contract)
[<img alt="crates.io" src="https://img.shields.io/crates/v/enum2contract.svg?style=for-the-badge&color=fc8d62&logo=rust" height="20">](https://crates.io/crates/enum2contract)
[<img alt="docs.rs" src="https://img.shields.io/badge/docs.rs-enum2contract-66c2a5?style=for-the-badge&labelColor=555555&logo=docs.rs" height="20">](https://docs.rs/enum2contract)


`enum2contract` is a no_std compatible rust derive macro that lets users specify contracts for pub/sub style messaging using strongly typed rust enums.

Conversion methods for JSON and binary are created for the generated payloads.

## Usage

Add this to your `Cargo.toml`:

```toml
enum2contract = "0.1.6"
serde = { version = "1.0.158", default-features = false, features = ["derive"] }
serde_json = { version = "1.0.94", default-features = false, features = ["alloc"] }
postcard = { version = "1.0.4", features = ["alloc"] }
```

Example:

```rust
use enum2contract::EnumContract;
use serde::{Deserialize, Serialize};

#[derive(EnumContract)]
pub enum Message {
    #[topic("notify/{group}")]
    Notify,

    #[topic("notify_all")]
    NotifyAll,

    #[topic("system/{id}/start/{mode}")]
    Start { immediate: bool, timeout: u64 },
}

#[test]
fn topic() {
    assert_eq!(Message::notify_topic("subset"), "notify/subset");

    assert_eq!(Message::notify_all_topic(), "notify_all");

    assert_eq!(
        Message::start_topic(&3.to_string(), "idle"),
        "system/3/start/idle"
    );
}

#[test]
fn message() {
    assert_eq!(
        Message::notify("subgroup"),
        ("notify/subgroup".to_string(), NotifyPayload::default())
    );

    assert_eq!(
        Message::notify_all(),
        ("notify_all".to_string(), NotifyAllPayload::default())
    );

    assert_eq!(
        Message::start(&3.to_string(), "idle"),
        ("system/3/start/idle".to_string(), StartPayload::default())
    );
}

#[test]
fn notify_payload_from_json_with_data() {
    let json = r#"{"immediate":true,"timeout":40}"#;
    let payload = StartPayload::from_json(json).unwrap();
    assert_eq!(
        payload,
        StartPayload {
            immediate: true,
            timeout: 40,
        }
    );
}
```

> This crate is `#![no_std]` compatible but requires `alloc`.
