use std::collections::HashMap;
use serde::{Serialize, Deserialize};
use serde_json::{to_value, to_vec, to_string, from_value, from_slice, from_str, Value};
use anyhow::anyhow;
use cloudevents::{
    AttributesReader,
    Data as CloudEventData,
    Event as CloudEvent,
    EventBuilder as CloudEventBuilder,
    EventBuilderV10 as CloudEventBuilderV10,
    event::{TryIntoTime, TryIntoUrl, ExtensionValue},
};
use url::Url;

use crate::error::{Error, Result};


/// Trait for event data types.
pub trait EventData: Serialize + for<'de> Deserialize<'de> + Send + Sync + Clone + 'static {
    /// Returns the event type as a static string.
    fn event_type() -> &'static str;
    /// Returns the channel name as a static string.
    fn channel_name() -> &'static str;
}

/// An empty event data type.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct EmptyEventData;

impl EventData for EmptyEventData {
    fn event_type() -> &'static str { "_" }
    fn channel_name() -> &'static str { "_" }
}


/// A type for containing an event's information.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Event(pub CloudEvent);

impl Event {
    pub fn new(event: CloudEvent) -> Self {
        Self(event)
    }

    /// Returns a new [`EventBuilder`].
    pub fn builder() -> EventBuilder {
        EventBuilder::default()
    }

    /// Returns a reference to the inner [`CloudEvent`].
    pub fn inner(&self) -> &CloudEvent {
        &self.0
    }

    /// Returns the [`CloudEvent`] spec version.
    pub fn specversion(&self) -> String {
        self.0.specversion().as_str().to_string()
    }

    /// Returns the event ID.
    pub fn id(&self) -> &str {
        self.0.id()
    }

    /// Returns the event source.
    pub fn source(&self) -> &str {
        self.0.source()
    }

    /// Returns the event type.
    pub fn type_(&self) -> &str {
        self.0.ty()
    }

    /// Returns the optional event time.
    pub fn time(&self) -> Option<&chrono::DateTime<chrono::Utc>> {
        self.0.time()
    }

    /// Returns the optional data content type.
    pub fn datacontenttype(&self) -> Option<&str> {
        self.0.datacontenttype()
    }

    /// Returns the optional data schema URL.
    pub fn dataschema(&self) -> Option<&Url> {
        self.0.dataschema()
    }

    /// Returns the optional event subject.
    pub fn subject(&self) -> Option<&str> {
        self.0.subject()
    }

    /// Returns a map of all extensions.
    pub fn extensions(&self) -> HashMap<String, ExtensionValue> {
        self.0.iter_extensions()
            .map(|(k, v)| (k.to_string(), v.clone()))
            .collect()
    }

    /// Returns the event data as serialized bytes.
    pub fn data_as_bytes(&self) -> Result<Vec<u8>> {
        match self.0
            .data()
            .ok_or(Error::MissingEventData)?
        {
            CloudEventData::Binary(bytes) => Ok(bytes.clone()),
            CloudEventData::Json(value) => to_vec(value)
                .map_err(|e| Error::Serialization(e.to_string())),
            CloudEventData::String(s) => Ok(s.as_bytes().to_vec()),
        }
    }

    /// Returns the event data as a [`serde_json::Value`].
    pub fn data_as_value(&self) -> Result<Value> {
        match self.0
            .data()
            .ok_or(Error::MissingEventData)?
        {
            CloudEventData::Json(value) => Ok(value.clone()),
            CloudEventData::Binary(bytes) => from_slice(&bytes)
                .map_err(|e| Error::Deserialization(e.to_string())),
            CloudEventData::String(s) => from_str(&s)
                .map_err(|e| Error::Deserialization(e.to_string())),
        }
    }

    /// Returns the event data as a JSON string.
    pub fn data_as_string(&self) -> Result<String> {
        match self.0
            .data()
            .ok_or(Error::MissingEventData)?
        {
            CloudEventData::String(s) => Ok(s.clone()),
            CloudEventData::Json(value) => serde_json::to_string(&value)
                .map_err(|e| Error::Serialization(e.to_string())),
            CloudEventData::Binary(bytes) => from_slice::<Value>(&bytes)
                .and_then(|v| to_string(&v))
                .map_err(|e| Error::Serialization(e.to_string())),
        }
    }

    /// Returns the event data deserialized into the specified type.
    pub fn data<E: EventData>(&self) -> Result<E> {
        match self.0
            .data()
            .ok_or(Error::MissingEventData)?
        {
            CloudEventData::Json(value) => from_value(value.clone())
                .map_err(|e| Error::Deserialization(e.to_string())),
            CloudEventData::Binary(bytes) => from_slice(&bytes)
                .map_err(|e| Error::Deserialization(e.to_string())),
            CloudEventData::String(s) => from_str(&s)
                .map_err(|e| Error::Deserialization(e.to_string())),
        }
    }

    /// Returns an empty event.
    pub fn empty() -> Self {
        EventBuilder::new()
            .id("_empty")
            .source("_empty")
            .build(EmptyEventData)
            .unwrap()
    }
}


pub struct EventBuilder {
    inner: CloudEventBuilderV10,
    schema_url: Option<String>,
    error: Option<Error>,
}

impl EventBuilder {
    pub fn new() -> Self {
        Self {
            inner: CloudEventBuilderV10::default(),
            schema_url: None,
            error: None,
        }
    }

    pub fn id(mut self, id: impl Into<String>) -> Self {
        self.inner = self.inner.id(id);
        self
    }

    pub fn maybe_id(mut self, id: Option<impl Into<String>>) -> Self {
        if let Some(id) = id {
            self.inner = self.inner.id(id);
        }

        self
    }

    pub fn source(mut self, source: impl Into<String>) -> Self {
        self.inner = self.inner.source(source);
        self
    }

    pub fn maybe_source(mut self, source: Option<impl Into<String>>) -> Self {
        if let Some(source) = source {
            self.inner = self.inner.source(source);
        }

        self
    }

    pub fn subject(mut self, subject: impl Into<String>) -> Self {
        self.inner = self.inner.subject(subject);
        self
    }

    pub fn maybe_subject(mut self, subject: Option<impl Into<String>>) -> Self {
        if let Some(subject) = subject {
            self.inner = self.inner.subject(subject);
        }

        self
    }

    pub fn time(mut self, time: impl TryIntoTime) -> Self {
        self.inner = self.inner.time(time);
        self
    }

    pub fn maybe_time(mut self, time: Option<impl TryIntoTime>) -> Self {
        if let Some(time) = time {
            self.inner = self.inner.time(time);
        }

        self
    }

    pub fn extension(mut self, name: &str, value: impl Into<ExtensionValue>) -> Self {
        self.inner = self.inner.extension(name, value);
        self
    }

    pub fn maybe_extension(mut self, name: &str, value: Option<impl Into<ExtensionValue>>) -> Self {
        if let Some(value) = value {
            self.inner = self.inner.extension(name, value);
        }

        self
    }

    pub fn extensions(mut self, extensions: HashMap<impl AsRef<str>, impl Into<ExtensionValue>>) -> Self {
        for (k, v) in extensions {
            self.inner = self.inner.extension(k.as_ref(), v);
        }
        self
    }

    pub fn maybe_extensions(mut self, extensions: Option<HashMap<impl AsRef<str>, Option<impl Into<ExtensionValue>>>>) -> Self {
        if let Some(exts) = extensions {
            for (k, v) in exts {
                if let Some(v) = v {
                    self.inner = self.inner.extension(k.as_ref(), v);
                }
            }
        }

        self
    }

    pub fn schema_url(mut self, schema_url: impl TryIntoUrl) -> Self {
        match schema_url.into_url() {
            Ok(url) => self.schema_url = Some(url.to_string()),
            Err(e) => {
                self.error = Some(Error::Unknown(anyhow!(e)));
            }
        };

        self
    }

    pub fn maybe_schema_url(mut self, schema_url: Option<impl TryIntoUrl>) -> Self {
        if let Some(schema_url) = schema_url {
            match schema_url.into_url() {
                Ok(url) => self.schema_url = Some(url.to_string()),
                Err(e) => {
                    self.error = Some(Error::Unknown(anyhow!(e)));
                }
            };
        }

        self
    }

    pub fn type_<T: AsRef<str>>(mut self, type_: T) -> Self {
        self.inner = self.inner.ty(type_.as_ref());
        self
    }

    pub fn maybe_type<T: AsRef<str>>(mut self, type_: Option<T>) -> Self {
        if let Some(type_) = type_ {
            self.inner = self.inner.ty(type_.as_ref());
        }

        self
    }

    pub fn build<E: EventData>(mut self, data: E) -> Result<Event> {
        let value = match to_value(&data) {
            Ok(v) => v,
            Err(e) => {
                return Err(Error::Serialization(e.to_string()));
            }
        };

        if let Some(err) = self.error.take() {
            return Err(err);
        }

        self.inner = match self.schema_url {
            Some(ref url) => self.inner.data_with_schema("application/json", url.to_string(), value),
            None => self.inner.data("application/json", value),
        };
        self.inner = self.inner.ty(E::event_type());

        Ok(
            Event::new(
                self.inner.build()
                    .map_err(|e| Error::Unknown(anyhow!(e)))?
            )
        )
    }

    pub fn build_raw(mut self, data: Vec<u8>) -> Result<Event> {
        if let Some(err) = self.error.take() {
            return Err(err);
        }

        self.inner = match self.schema_url {
            Some(ref url) => self.inner.data_with_schema("application/json", url.to_string(), data),
            None => self.inner.data("application/json", data),
        };

        Ok(
            Event::new(
                self.inner.build()
                    .map_err(|e| Error::Unknown(anyhow!(e)))?
            )
        )
    }
}

impl Default for EventBuilder {
    fn default() -> Self {
        Self::new()
    }
}