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
Snowplow Data Types. The types in this module implement various parts of the
Snowplow protocol data model, and in particular its outmost event type and
the `{schema, data}` pairs that are common to custom user data.

Generally you should focus on the types and traits that are part of the
[`Tracker`][crate::tracker::Tracker] interface; the other types in this library
are only for very custom or very advanced use cases.
*/

use std::fmt;
use std::fmt::Display;
use std::fmt::Formatter;
use std::time::SystemTime;

use serde::ser::SerializeStruct as _;
use serde::{Serialize, Serializer};

use crate::util::JsonString;
use crate::util::Stringify;

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

/// The event type we're sending. Currently we only support "self-describing"
/// events.
#[derive(Default, Serialize, Clone, Copy, Debug)]
pub enum EventType {
    /// An unstructured event, described by a schema.
    #[default]
    #[serde(rename(serialize = "ue"))]
    SelfDescribingEvent,
}

/// The platform this tracker is being used on. This is generally fixed at
/// compile time, but this library is broadly cross-platform, so it still needs
/// to be provided during [`Tracker`][crate::Tracker] configuration.
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

    /// Smart TV
    #[serde(rename = "tv")]
    Tv,

    /// Video game console
    #[serde(rename = "cnsl")]
    GameConsole,

    /// Internet of Things device
    #[serde(rename = "iot")]
    Thing,
}

/// A snowplow timestamp. Serializes as the number of seconds since the unix
/// epoch.
///
/// Note that this type serializes as a string, even though it has numeric
/// representation, in keeping with the Snowplow Protocol. You should probably
/// not use it for your own custom event timekeeping.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct SnowplowTimestamp {
    timestamp: SystemTime,
}

impl SnowplowTimestamp {
    /// Create a new timestamp for this moment in time. This is based on
    /// [`std::time::SystemTime`].
    pub fn now() -> Self {
        Self {
            timestamp: SystemTime::now(),
        }
    }
}

impl Serialize for SnowplowTimestamp {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        // TODO: log a warning. Bring in tracing for general logging.
        let timestamp_millis = self
            .timestamp
            .duration_since(SystemTime::UNIX_EPOCH)
            .map(|duration| duration.as_millis())
            .unwrap_or(0);

        let mut buffer = itoa::Buffer::new();
        let formatted = buffer.format(timestamp_millis);

        serializer.serialize_str(formatted)
    }
}

/// The data type for a Snowplow event. Generally you won't need to create
/// `SnowplowEvent` objects directly; you should prefer instead to create
/// [`TrackedEvent`][crate::tracker::TrackedEvent] objects. See the
/// [`Tracker`][crate::tracker::Tracker] for details.
#[derive(Serialize, Clone, Debug)]
pub struct SnowplowEvent<'a, Payload: HasSchema> {
    // ----- PAYLOAD ------
    // TODO: replace this with an enum that handles the variations
    /// The event type; currently we only support self-describing events
    #[serde(rename = "e")]
    pub event_type: EventType,

    /// The user
    #[serde(rename = "ue_pr")]
    pub payload: JsonString<PayloadWrapper<Payload>>,

    // ------ APPLICATION PARAMETERS ------
    /// The platform that this tracker is being used on
    #[serde(rename = "p")]
    pub platform: Platform,

    /// An identifier describing this app
    #[serde(rename = "aid")]
    pub app_id: &'a str,

    /// The name of the tracker. This should generally always be the name &
    /// version of this Rust crate
    #[serde(rename = "tv")]
    pub tracker_id: &'static str,

    /// An identifier describing this specific tracker in the context of the
    /// application. If your application is using multiple trackers, this field
    /// distinguishes between them.
    #[serde(rename = "tna")]
    pub namespace: &'a str,

    // ----- GENERIC EVENT META ------
    /// The ID for this event. If omitted, one will be generated by the
    /// snowplow collector. Generally you only need to set this if there's a
    /// risk of duplicate events getting sent to the collector.
    #[serde(rename = "eid")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub event_id: Option<uuid::Uuid>,

    /// The timestamp at which this event occurred.
    #[serde(rename = "dtm")]
    pub created_timestamp: SnowplowTimestamp,

    /// The timestamp at which this event was sent to a collector. This should
    /// be populated by the emitter at the moment the event is sent.
    #[serde(rename = "stm")]
    pub sent_timestamp: SnowplowTimestamp,
}

/// An Iglu Schema version. Renders as `{major}-{minor}-{patch}`
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(missing_docs)]
pub struct SchemaVersion {
    pub major: u32,
    pub minor: u32,
    pub patch: u32,
}

impl SchemaVersion {
    /// Create a new Snowplow schema version, of the form
    /// `{major}-{minor}-{patch}`.
    pub fn new(major: u32, minor: u32, patch: u32) -> Self {
        Self {
            major,
            minor,
            patch,
        }
    }
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

/// An Iglu Schema. Renders as `iglu:{vendor}/{name}/jsonschema/{version}`
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Schema {
    /// Typically a reverse domain name, like "com.agilebits.desktop"
    pub vendor: &'static str,

    /// The name of this specific schema
    pub name: &'static str,

    /// The version of this specific schema
    pub version: SchemaVersion,
}

impl Schema {
    /// Build a new schema. This will resemble
    /// "`iglu:{vendor}/{name}/jsonschema/{version}`". Schemas tend to be fixed
    /// to a particular type, so all the string components are `&'static str`
    // TODO: macro version of this constructor so that the entire schema
    // can been build-time concatenated as a single string. This would provide
    // an opportunity for build time verification of the various components.
    #[inline]
    #[must_use]
    pub fn new(vendor: &'static str, name: &'static str, version: SchemaVersion) -> Self {
        Self {
            vendor,
            name,
            version,
        }
    }

    /// Build a new schema where the vendor is `com.snowplowanalytics.snowplow`.
    /// For now this is internal-only; if library consumers want to use a
    /// Snowplow first-party schema, we still ask them to be explicit and
    /// supply the vendor themselves, for consistency.
    #[inline]
    pub(crate) fn new_snowplow(name: &'static str, version: SchemaVersion) -> Self {
        Self::new("com.snowplowanalytics.snowplow", name, version)
    }
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

/// Catch-all type for the snowplow data envelope, which combines a snowplow
/// schema ID with some kind of payload. The payload includes the schema via
/// the [`HasSchema`] trait. The [`Envelope`] will serialize as an object
/// resembling `{"schema": "SCHEMA", "data": data}`.
#[derive(Debug, Clone, Copy, Default)]
pub struct Envelope<T: HasSchema>(
    /// The custom data for the event.
    ///
    /// The schema for this data is a part of the data's type, via [`HasSchema`]
    pub T,
);

/// Trait for types that have a Snowplow Schema. See [`Envelope`] for details.
pub trait HasSchema {
    /// Get the schema associated with this type.
    fn schema(&self) -> Schema;
}

impl<T: HasSchema + Serialize> Serialize for Envelope<T> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let data = &self.0;
        let mut map = serializer.serialize_struct("Envelope", 2)?;
        map.serialize_field("schema", &Stringify(data.schema()))?;
        map.serialize_field("data", &data)?;
        map.end()
    }
}

/**
Snowplow imposes a *lot* of nesting on the way that event playloads are sent.
A typical event payload resembles:

```json
{
  "schema": "iglu:com.snowplowanalytics.snowplow/unstruct_event/jsonschema/1-0-0",
  "data": {
    "schema": "iglu:com.my_company/viewed_product/jsonschema/1-0-0",
    "data": {
      "product_id": "ASO01043",
      "price": 49.95
    }
  }
}
```

This type handles this nested wrapper. The innermost `Payload` type corresponds
to the `{"product_id", "price"}` struct above, which should implement
[`HasSchema`]; the remaining nesting (including the outer snowplow schema) is
handled here.
*/
pub type PayloadWrapper<Payload> = Envelope<UnstructWrapper<Envelope<Payload>>>;

/// An [`UnstructWrapper`] corresponds to the outer "data" envelope of the
/// Snowplow Unstructured Event layout. It mostly exists to supply the
/// `"iglu:com.snowplowanalytics.snowplow/unstruct_event/jsonschema/1-0-0"`
/// schema via [`HasSchema`]. Generally you won't need to deal with this type
/// directly.
#[derive(Debug, Clone, Default, Copy, Serialize)]
#[serde(transparent)]
pub struct UnstructWrapper<Payload>(pub Payload);

impl<Payload> HasSchema for UnstructWrapper<Payload> {
    fn schema(&self) -> Schema {
        Schema::new_snowplow("unstruct_event", SchemaVersion::new(1, 0, 0))
    }
}

impl<Payload: HasSchema> PayloadWrapper<Payload> {
    /// Create a new `PayloadWrapper`. This constructor handles all the nesting
    /// of [`Envelope`] etc.
    pub fn new(payload: Payload) -> Self {
        Envelope(UnstructWrapper(Envelope(payload)))
    }
}
