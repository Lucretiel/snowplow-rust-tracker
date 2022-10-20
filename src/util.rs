/*!
Utility types for serialization. This module contains serde serialization
wrappers that manage the stringification that Snowplow demands for its
event containers.
*/

use std::cell::Cell;
use std::fmt::{Display, Write as _};

use lazy_format::lazy_format;
use serde::ser;
use serde_json::to_string;

thread_local! {
    static STRINGIFY_BUFFER: Cell<String> = Cell::new(String::new());
}

/// Adapter type that serializes something by converting it into a string and
/// serializing that. Useful for primitive types like ints.
#[derive(Debug, Clone, Copy, Default)]
pub struct Stringify<T>(pub T);

impl<T: Display> ser::Serialize for Stringify<T> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        STRINGIFY_BUFFER.with(|cell| {
            // Take temporary ownership of the buffer. This will replace it
            // with an empty buffer in the thread local
            let mut buffer = cell.take();

            // Stringify the value
            buffer.clear();
            write!(&mut buffer, "{}", self.0).expect("write! to a string is infallible");

            // Serialize the string
            let res = serializer.serialize_str(&buffer);

            // Restore the buffer to the thread_local for future use
            cell.set(buffer);
            res
        })
    }
}

/// Adapter type that serializes something by first converting it to a JSON string
/// and serializing that as a string. For some reason.
#[derive(Debug, Clone, Copy, Default)]
pub struct JsonString<T>(pub T);

impl<T: ser::Serialize> ser::Serialize for JsonString<T> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let jsonified = to_string(&self.0).map_err(|json_err| {
            ser::Error::custom(lazy_format!("Error serializing to JSON string: {json_err}"))
        })?;

        serializer.serialize_str(&jsonified)
    }
}
