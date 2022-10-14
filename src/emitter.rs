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

use std::future::ready;

use futures::TryStreamExt as _;
use reqwest::Client;
use serde::Serialize;

use url::Url;

use crate::payload::Payload;

#[derive(Debug, Clone, Serialize)]
struct PayloadContainer {
    schema: &'static str,
    data: Vec<Payload>,
}

/// Emitter is responsible for emitting tracked events to the Snowplow
/// Collector. It takes care of the low-level HTTP stuff.
pub struct Emitter {
    collector_url: Url,
    client: Client,
}

impl Emitter {
    pub fn new(collector_url: Url, client: Client) -> Emitter {
        Emitter {
            collector_url,
            client,
        }
    }

    pub async fn track_events(
        &self,
        events: impl IntoIterator<Item = Payload>,
    ) -> Result<(), reqwest::Error> {
        let events = PayloadContainer {
            schema: "iglu:com.snowplowanalytics.snowplow/payload_data/jsonschema/1-0-4",
            data: events.into_iter().collect(),
        };

        let response = self
            .client
            .post(self.collector_url.clone())
            .json(&events)
            .send()
            .await?;

        // Snowplow responses don't contain anything useful, so just drain the
        // response content.
        response
            .bytes_stream()
            .try_for_each(|_chunk| ready(Ok(())))
            .await
    }

    pub async fn track_event(&self, event: Payload) -> Result<(), reqwest::Error> {
        self.track_events([event]).await
    }
}
