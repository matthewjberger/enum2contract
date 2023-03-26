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
