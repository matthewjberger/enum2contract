# enum2contract

[<img alt="github" src="https://img.shields.io/badge/github-matthewjberger/enum2contract-8da0cb?style=for-the-badge&labelColor=555555&logo=github" height="20">](https://github.com/matthewjberger/enum2contract)
[<img alt="crates.io" src="https://img.shields.io/crates/v/enum2contract.svg?style=for-the-badge&color=fc8d62&logo=rust" height="20">](https://crates.io/crates/enum2contract)
[<img alt="docs.rs" src="https://img.shields.io/badge/docs.rs-enum2contract-66c2a5?style=for-the-badge&labelColor=555555&logo=docs.rs" height="20">](https://docs.rs/enum2contract)


enum2contract is a no_std compatible rust derive macro that lets users specify contracts for pub/sub style messaging using strongly typed rust enums.

## Usage

Add this to your `Cargo.toml`:

```toml
enum2contract = "0.1.1"
```

Example:

```rust
use enum2contract::EnumContract;

#[derive(EnumContract)]
pub enum Message {
    #[topic("notify/{group}")]
    Notify,

    #[topic("notify_all")]
    NotifyAll,

    #[topic("system/{id}/start/{mode}")]
    Start { immediate: bool },
}

#[test]
fn create_notify_message() {
    let (topic, payload) = Message::notify("partial");
    assert_eq!(topic, "notify/partial");
    assert_eq!(payload, NotifyPayload::default());
}

#[test]
fn create_notify_all_message() {
    let (topic, payload) = Message::notify_all();
    assert_eq!(topic, "notify_all");
    assert_eq!(payload, NotifyAllPayload::default());
}

#[test]
fn create_start_message() {
    let (topic, payload) = Message::start("76", "idle");
    assert_eq!(topic, "system/76/start/idle");
    assert_eq!(payload, StartPayload { immediate: false });
}
```

> This crate is `#![no_std]` compatible!
