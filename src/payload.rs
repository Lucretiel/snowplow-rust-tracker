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

use std::fmt;
use std::fmt::Display;
use std::fmt::Formatter;

use serde::{Deserialize, Serialize};
use serde_json::json;
use serde_json::Value;

/// Wrapper that causes the internal type to be serialized
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StringySerde<T>(pub T);

impl<T: ToString> Serialize for StringySerde<T> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        // TODO: reusable thread-local buffer or something like that
        serializer.serialize_str(&self.0.to_string())
    }
}

#[derive(Default, Serialize, Clone, Copy, Debug)]
pub enum EventType {
    #[default]
    #[serde(rename(serialize = "ue"))]
    SelfDescribingEvent,
}

#[derive(Debug, Default, Serialize, Clone, Copy)]
pub enum Platform {
    /// Websites
    #[serde(rename = "web")]
    Web,

    /// Mobile apps (for smartphones, tablets, etc)
    #[serde(rename = "mob")]
    Mobile,

    /// Native desktop apps used by consumers
    #[serde(rename = "pc")]
    Desktop,

    /// Applications used on servers, like web servers, databases, etc
    #[serde(rename = "srv")]
    ServerSide,

    /// General app
    #[default]
    #[serde(rename = "app")]
    App,
    // TODO: iot, tv, cnsl
}

// TODO: use a real timekeeping crate and a real timestamp in this struct
#[derive(Debug, Copy, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
struct SnowplowTimestamp {
    // a snowplow timestamp is "unix time but in milliseconds". 64 bits is
    // roughly 1 order of magnitude short of being able to express the
    // estimated age of the universe, so we assume it's fine for our purposes.
    timestamp: i64,
}

#[derive(Serialize, Default, Clone, Debug)]
pub struct Payload {
    // TODO: replace this with an enum that handles the variations
    #[serde(rename = "e")]
    pub event_type: EventType,

    #[serde(rename = "p")]
    pub platform: Platform,

    /// An identifier describing this app
    #[serde(rename = "aid")]
    pub app_id: String,

    /// The name of the tracker. This should generally always be the name &
    /// version of this Rust crate
    #[serde(rename = "tv")]
    pub tracker_id: &'static str,

    #[serde(rename = "eid")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub eid: Option<uuid::Uuid>,

    /// The timestamp at which this event occurred.
    #[serde(rename = "dtm")]
    created_timestamp: SnowplowTimestamp,

    /// The timestamp at which this event was sent to a collector. This should
    /// be populated by the emitter at the moment the event is sent.
    #[serde(rename = "stm")]
    sent_timestamp: SnowplowTimestamp,
    // TODO: Context
    // TODO: payload
}

/// An Iglu Schema version. Renders as `"{major}-{minor}-{patch}"`
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct SchemaVersion {
    pub major: u32,
    pub minor: u32,
    pub patch: u32,
}

impl Display for SchemaVersion {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let Self {
            major,
            minor,
            patch,
        } = *self;

        write!(f, "{major}-{minor}-{patch}")
    }
}

/// An Iglu Schema. Renders as "`iglu:{vendor}/{name}/jsonschema/{version}`"
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Schema {
    /// Typically a reverse domain name, like "com.agilebits.desktop"
    pub vendor: &'static str,

    /// The name of this specific schema
    pub name: &'static str,

    /// The version of this specific schema
    pub version: SchemaVersion,
}

impl Display for Schema {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let Self {
            vendor,
            name,
            version,
        } = *self;

        write!(f, "iglu:{vendor}/{name}/jsonschema/{version}")
    }
}

impl Serialize for Schema {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.collect_str(self)
    }
}

/// Catch-all type for the snowplow data envelope, which combines a snowplow
/// schema ID with some kind of payload
#[derive(Serialize, Deserialize)]
pub struct Envelope<T> {
    /// A valid Iglu schema path.
    ///
    /// This must point to the location of the custom event’s schema, of the
    /// format: `iglu:{vendor}/{name}/{format}/{version}`.
    pub schema: Schema,

    /// The custom data for the event.
    ///
    /// This data must conform to the schema specified in the schema argument,
    /// or the event will fail validation and land in bad rows.
    pub data: T,
}
