use enum2contract::EnumContract;
use serde::{Deserialize, Serialize};

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
fn notify_message() {
    let (topic, payload) = Message::notify("partial");
    assert_eq!(topic, "notify/partial");
    assert_eq!(payload, NotifyPayload::default());
}

#[test]
fn notify_all_message() {
    let (topic, payload) = Message::notify_all();
    assert_eq!(topic, "notify_all");
    assert_eq!(payload, NotifyAllPayload::default());
}

#[test]
fn start_message() {
    let (topic, payload) = Message::start("76", "idle");
    assert_eq!(topic, "system/76/start/idle");
    assert_eq!(payload, StartPayload { immediate: false });
}

#[test]
fn notify_payload_json_conversion() {
    let payload = NotifyPayload;
    let json = payload.to_json().unwrap();
    assert_eq!(json, "null");

    let payload2: NotifyPayload = NotifyPayload::from_json(&json).unwrap();
    assert_eq!(payload2, payload);
}

#[test]
fn notify_all_payload_json_conversion() {
    let payload = NotifyAllPayload;
    let json = payload.to_json().unwrap();
    assert_eq!(json, "null");

    let payload2: NotifyAllPayload = NotifyAllPayload::from_json(&json).unwrap();
    assert_eq!(payload2, payload);
}

#[test]
fn start_payload_json_conversion() {
    let payload = StartPayload { immediate: true };
    let json = payload.to_json().unwrap();
    assert_eq!(json, r#"{"immediate":true}"#);

    let payload2: StartPayload = StartPayload::from_json(&json).unwrap();
    assert_eq!(payload2, payload);
}

#[test]
fn notify_payload_from_json() {
    let json = "null";
    let payload = NotifyPayload::from_json(json).unwrap();
    assert_eq!(payload, NotifyPayload::default());
}

#[test]
fn notify_payload_from_json_with_data() {
    let json = r#"{"immediate": true}"#;
    let payload = StartPayload::from_json(json).unwrap();
    assert_eq!(payload, StartPayload { immediate: true });
}
