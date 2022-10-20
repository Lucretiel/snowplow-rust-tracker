// Copyright (c) 2022 Snowplow Analytics Ltd. All rights reserved.
//
// This program is licensed to you under the Apache License Version 2.0,
// and you may not use this file except in compliance with the Apache License Version 2.0.
// You may obtain a copy of the Apache License Version 2.0 at http://www.apache.org/licenses/LICENSE-2.0.
//
// Unless required by applicable law or agreed to in writing,
// software distributed under the Apache License Version 2.0 is distributed on an
// "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the Apache License Version 2.0 for the specific language governing permissions and limitations there under.

use serde::Serialize;
use snowplow_tracker::{HasSchema, Platform, Schema, SchemaVersion, TrackedEvent, Tracker};
use uuid::Uuid;

// An example unstructured event we might want to track
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

#[tokio::main]
async fn main() {
    let tracker = Tracker::build(
        "ns",
        "app_id".to_owned(),
        Platform::Desktop,
        "http://localhost:9090/com.snowplowanalytics.snowplow/tp2"
            .parse()
            .expect("hardcoded URL"),
        reqwest::Client::new(),
    );

    let event_id = Uuid::new_v4();

    tracker
        .track(TrackedEvent {
            payload: WebPage {
                name: "test".to_owned(),
                id: "something else".to_owned(),
            },
            id: Some(event_id),
            timestamp: None,
        })
        .await
        .expect("Failed to send Snowplow event");

    println!("--- DEBUGGING ---");
    println!("Self Describing Event: {}", event_id);
}
