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

/*!
A snowplow event [`Emitter`]. This type manages the low-level details of sending
events over HTTP to a Collector. Generally you should prefer to use a
[`Tracker`][crate::tracker::Tracker], which wraps an [`Emitter`] handles a lot
of the bookkeeping required to construct full snowplow events.
 */

use std::future::ready;

use futures::TryStreamExt as _;
use reqwest::Client;

use serde::Serialize;
use url::Url;

use crate::payload::{Envelope, HasSchema, Schema, SchemaVersion, SnowplowEvent};

/// The outermost type that is actually sent to snowplow as a JSON payload.
/// Includes an outermost schema and a Vec of [`SnowplowEvent`].
// TODO: It will be exceedingly common to only need to send a single event;
// create an optimized version of this type to handle that use case.
pub(crate) type EventContainer<'a, Payload> = Envelope<Vec<SnowplowEvent<'a, Payload>>>;

impl<'a, Payload: HasSchema> EventContainer<'a, Payload> {
    /// Create a new event container. This will collect all of the given events
    /// into a [`Vec`].
    pub fn new(events: impl IntoIterator<Item = SnowplowEvent<'a, Payload>>) -> Self {
        Envelope(events.into_iter().collect())
    }
}

impl<'a, Payload: HasSchema> HasSchema for Vec<SnowplowEvent<'a, Payload>> {
    fn schema(&self) -> Schema {
        Schema::new_snowplow("payload_data", SchemaVersion::new(1, 0, 4))
    }
}

/// Emitter is responsible for emitting tracked events to the Snowplow
/// Collector. It takes care of the low-level HTTP stuff. You should probably
/// be using [`Tracker`][crate::Tracker] instead.
pub struct Emitter {
    collector_url: Url,
    client: Client,
}

impl Emitter {
    /// Create a new emitter that will send events to the given Url using the
    /// given client.
    pub const fn new(collector_url: Url, client: Client) -> Emitter {
        // TODO: log a warning if the Url doesn't look right
        Emitter {
            collector_url,
            client,
        }
    }

    /// Track a batch of events, sending them to the snowplow collector
    pub async fn track_events<Payload: HasSchema + Serialize>(
        &self,
        events: impl IntoIterator<Item = SnowplowEvent<'_, Payload>>,
    ) -> Result<(), reqwest::Error> {
        let events = EventContainer::new(events);

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

    /// Track a single event
    pub async fn track_event<Payload: HasSchema + Serialize>(
        &self,
        event: SnowplowEvent<'_, Payload>,
    ) -> Result<(), reqwest::Error> {
        self.track_events([event]).await
    }
}
