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
    let (topic, mut payload) = Message::start("76", "idle");
    payload.timeout = 100;
    assert_eq!(topic, "system/76/start/idle");
    assert_eq!(
        payload,
        StartPayload {
            immediate: false,
            timeout: 100
        }
    );
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
    let payload = StartPayload {
        immediate: true,
        timeout: 321,
    };
    let json = payload.to_json().unwrap();
    assert_eq!(json, r#"{"immediate":true,"timeout":321}"#);

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
