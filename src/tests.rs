#[cfg(test)]
mod tests {
    use crate::emitter::EventContainer;
    use crate::{
        payload::{EventType, PayloadWrapper, SnowplowEvent, SnowplowTimestamp},
        util::JsonString,
        HasSchema, Platform, Schema, SchemaVersion, TrackedEvent,
    };
    use serde::Serialize;
    use serde_test::{assert_ser_tokens, Configure, Token};
    use std::time::SystemTime;
    use uuid::Uuid;

    #[derive(Debug, Serialize)]
    struct WebPage {
        name: String,
        id: String,
    }

    impl HasSchema for WebPage {
        fn schema(&self) -> Schema {
            Schema::new(
                "com.snowplowanalytics.snowplow",
                "screen_view",
                SchemaVersion::new(1, 0, 0),
            )
        }
    }

    #[test]
    fn test_envelope_serialization() {
        let test_payload = WebPage {
            name: "test".to_owned(),
            id: "test id".to_owned(),
        };
        let wrapper = PayloadWrapper::new(test_payload);
        assert_ser_tokens(
            &wrapper,
            &[
                Token::Struct {
                    name: "Envelope",
                    len: 2,
                },
                Token::Str("schema"),
                Token::Str("iglu:com.snowplowanalytics.snowplow/unstruct_event/jsonschema/1-0-0"),
                Token::Str("data"),
                Token::Struct {
                    name: "Envelope",
                    len: 2,
                },
                Token::Str("schema"),
                Token::Str("iglu:com.snowplowanalytics.snowplow/screen_view/jsonschema/1-0-0"),
                Token::Str("data"),
                Token::Struct {
                    name: "WebPage",
                    len: 2,
                },
                Token::Str("name"),
                Token::Str("test"),
                Token::Str("id"),
                Token::Str("test id"),
                Token::StructEnd,
                Token::StructEnd,
                Token::StructEnd,
            ],
        );
    }

    #[test]
    fn test_emitter_event_construction() {
        let test_payload = WebPage {
            name: "test".to_owned(),
            id: "test id".to_owned(),
        };
        let mut test_event = TrackedEvent::new(test_payload);
        let test_uuid = Uuid::parse_str("a1a2a3a4b1b2c1c2d1d2d3d4d5d6d7d8");
        match test_uuid {
            Ok(test_uuid) => {
                let time_since_epoch;
                let current_timestamp = SystemTime::now();
                match current_timestamp.duration_since(SystemTime::UNIX_EPOCH) {
                    Ok(duration) => {
                        test_event.timestamp = Some(SnowplowTimestamp::from(current_timestamp));
                        time_since_epoch = duration.as_millis();
                    }
                    Err(_) => panic!("SystemTime before UNIX EPOCH!"),
                }

                let event_timestamp: &'static str =
                    Box::leak(time_since_epoch.to_string().into_boxed_str());

                test_event.id = Some(test_uuid.clone());
                let now = SnowplowTimestamp::now();
                let events = [test_event].into_iter().map(|event| SnowplowEvent {
                    event_type: EventType::SelfDescribingEvent,
                    payload: JsonString(PayloadWrapper::new(event.payload)),
                    platform: Platform::Desktop,
                    app_id: "test id",
                    tracker_id: "test tracker ID",
                    namespace: "test namespace",
                    event_id: event.id,
                    created_timestamp: event.timestamp.unwrap_or(now),
                    sent_timestamp: event.timestamp.unwrap_or(now),
                });

                let events = EventContainer::new(events);
                assert_ser_tokens(
                    &events.readable(),
                    &[
                        Token::Struct {
                            name: "Envelope",
                            len: 2,
                        },
                        Token::Str("schema"),
                        Token::Str("iglu:com.snowplowanalytics.snowplow/payload_data/jsonschema/1-0-4"),
                        Token::Str("data"),
                        Token::Seq { len: Some(1), },
                        Token::Struct { name: "SnowplowEvent", len: 9, },
                        Token::Str("e"),
                        Token::UnitVariant { name: "EventType", variant: "ue", },
                        Token::Str("ue_pr"),
                        Token::Str("{\"schema\":\"iglu:com.snowplowanalytics.snowplow/unstruct_event/jsonschema/1-0-0\",\"data\":{\"schema\":\"iglu:com.snowplowanalytics.snowplow/screen_view/jsonschema/1-0-0\",\"data\":{\"name\":\"test\",\"id\":\"test id\"}}}"),
                        Token::Str("p"),
                        Token::UnitVariant { name: "Platform", variant: "pc", },
                        Token::Str("aid"),
                        Token::Str("test id"),
                        Token::Str("tv"),
                        Token::Str("test tracker ID"),
                        Token::Str("tna"),
                        Token::Str("test namespace"),
                        Token::Str("eid"),
                        Token::Some,
                        Token::Str("a1a2a3a4-b1b2-c1c2-d1d2-d3d4d5d6d7d8"),
                        Token::Str("dtm"),
                        Token::Str(event_timestamp),
                        Token::Str("stm"),
                        Token::Str(event_timestamp),
                        Token::StructEnd,
                        Token::SeqEnd,
                        Token::StructEnd,
                    ]
                );
            }
            Err(_) => println!("Something went wrong when parsing the UUID"),
        }
    }
}
