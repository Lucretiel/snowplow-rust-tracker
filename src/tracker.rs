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
The main snowplow [`Tracker`], which used to send your events to a Collector.
The types in this module are your main entry point to this library.
*/

use serde::Serialize;
use std::fmt::Debug;
use thiserror::Error;
use url::Url;
use uuid::Uuid;

use crate::{
    emitter::Emitter,
    payload::{EventType, HasSchema, PayloadWrapper, Platform, SnowplowEvent, SnowplowTimestamp},
    util::JsonString,
};

/// An error encountered when submitting an event for tracking. Generally
/// collectors don't report issues when submitting unexpected
#[derive(Debug, Error)]
pub enum TrackError {
    /// There was an HTTP error sending the event– the response was malformed,
    /// or there was a TCP error. This variant does *not* include HTTP error
    /// codes.
    #[error("Unexpected error during HTTP request (not an error code)")]
    HttpConnection(#[from] reqwest::Error),
}

/// The tracker ID, corresponding to the `tv` field of a snowplow event.
/// This is deterministically set at compilation time.
///
/// TODO: use an environment variable to allow for build-time setting of the
/// tracker ID
const TRACKER_ID: &str = concat!("rust-fork-", env!("CARGO_PKG_VERSION"));

/// A [`TrackerConfig`] describes fields that are "global" to a particular
/// tracker instance and are attached to every event that is sent by the
/// tracker.
#[derive(Debug, Clone)]
pub struct TrackerConfig {
    /// The namespace this tracker will use
    pub namespace: &'static str,

    /// The platform we're operating on. If unsure, App is a good default.
    pub platform: Platform,

    /// An identifier for this specific application
    pub app_id: String,
}

/// Snowplow tracker instance used to track events to the Snowplow Collector.
///
/// The main purpose of the tracker is to build full snowplow event objects
/// out of the [`TrackedEvent`] objects you pass into it. It takes care of
/// stuff like event type, timestamps, app & tracker IDs, etc.
pub struct Tracker {
    /// Emitter used to send events to the Collector
    emitter: Emitter,
    /// Additional tracker config
    config: TrackerConfig,
}

impl Tracker {
    /// Create a new tracker directly out of its constituent parts
    ///
    /// Note that a snowplow collector URL usually has
    /// `'/com.snowplowanalytics.snowplow/tp2'` as its path. Unlike most
    /// snowplow trackers, we as you to include the full path, in case you want
    /// to change it for your specific collector configuration.
    pub fn build(
        namespace: &'static str,
        app_id: String,
        platform: Platform,
        url: Url,
        client: reqwest::Client,
    ) -> Self {
        Self::new(
            Emitter::new(url, client),
            TrackerConfig {
                namespace,
                platform,
                app_id,
            },
        )
    }

    /// Create a new tracker
    pub fn new(emitter: Emitter, config: TrackerConfig) -> Tracker {
        Tracker { emitter, config }
    }

    /// Tracks a Snowplow event and send it to the Snowplow collector.
    pub async fn track<Payload: HasSchema + Serialize>(
        &self,
        event: TrackedEvent<Payload>,
    ) -> Result<(), TrackError> {
        self.track_batch([event]).await
    }

    /// Track a batch of events, sending them to the snowplow collector.
    pub async fn track_batch<Payload: HasSchema + Serialize>(
        &self,
        events: impl IntoIterator<Item = TrackedEvent<Payload>>,
    ) -> Result<(), TrackError> {
        let now = SnowplowTimestamp::now();

        let events = events.into_iter().map(|event| SnowplowEvent {
            event_type: EventType::SelfDescribingEvent,
            payload: JsonString(PayloadWrapper::new(event.payload)),
            platform: self.config.platform,
            app_id: &self.config.app_id,
            tracker_id: TRACKER_ID,
            namespace: self.config.namespace,
            event_id: event.id,
            created_timestamp: event.timestamp.unwrap_or(now),
            sent_timestamp: now,
        });

        self.emitter
            .track_events(events)
            .await
            .map_err(TrackError::HttpConnection)
    }
}

/// An event to be sent to the tracker. Mostly this is a vehicle for your
/// Unstructured payload, but also allows you to include your own fields for
/// the top-level snowplow event
#[derive(Debug, Clone, Default)]
pub struct TrackedEvent<T: HasSchema + Serialize> {
    /// Your specific event payload. The tracker will handle correctly wrapping
    /// and encoding this according to the Snowplow protocol, so all you need
    /// to provide is your own data.
    ///
    /// The payload needs to implement `Serialize`, so it can be encoded as a
    /// JSON object, and it needs to implement `HasSchema` with a Snowplow
    /// schema ID corresponding to its layout.
    pub payload: T,

    /// The event Uuid. If omitted, one will be generated by the snowplow
    /// collector. You only need to populate this if your batching scheme might
    /// retry sending events and risk duplication
    pub id: Option<Uuid>,

    /// The moment when this event occurred. If omitted, we will use the moment
    /// that `track` is called. It's generally only necessary to fill this if
    /// your batching scheme imposes delay between when the event occurs and
    /// when it's tracked.
    pub timestamp: Option<SnowplowTimestamp>,
    // TODO: Contexts
}

impl<T: HasSchema + Serialize> TrackedEvent<T> {
    /// Create a new [`TrackedEvent`] with default event properties.
    pub fn new(payload: T) -> Self {
        Self {
            payload,
            id: None,
            timestamp: None,
        }
    }
}
