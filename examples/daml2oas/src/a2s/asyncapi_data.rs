use std::collections::BTreeMap;

use serde::Serialize;

use crate::schema::Schema;

const ASYNC_API_VERSION: &str = "2.0.0";
const WEB_SOCKET_CHANNEL_BINDINGS_VERSION: &str = "0.1.0";

#[derive(Debug, Serialize)]
pub struct AsyncAPI {
    pub asyncapi: String,
    pub info: Info,
    pub servers: Servers,
    pub channels: Channels,
    pub components: Components,
}

impl AsyncAPI {
    pub fn new(info: Info, servers: Servers, channels: Channels, components: Components) -> Self {
        Self {
            asyncapi: ASYNC_API_VERSION.to_string(),
            info,
            servers,
            channels,
            components,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct Info {
    pub title: String,
    pub version: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

impl Info {
    pub const fn new(title: String, version: String, description: Option<String>) -> Self {
        Self {
            title,
            version,
            description,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct Servers {
    #[serde(flatten)]
    pub servers: BTreeMap<String, Server>,
}

impl Servers {
    pub fn new(servers: BTreeMap<String, Server>) -> Self {
        Self {
            servers,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct Server {
    pub url: String,
    pub protocol: String,
}

impl Server {
    pub const fn new(url: String, protocol: String) -> Self {
        Self {
            url,
            protocol,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct Channels {
    #[serde(flatten)]
    pub items: BTreeMap<String, ChannelItem>,
}

impl Channels {
    pub fn new(items: BTreeMap<String, ChannelItem>) -> Self {
        Self {
            items,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct ChannelItem {
    pub description: String,
    pub publish: Operation,
    pub subscribe: Operation,
    pub bindings: ChannelBindings,
}

impl ChannelItem {
    pub const fn new(description: String, publish: Operation, subscribe: Operation, bindings: ChannelBindings) -> Self {
        Self {
            description,
            publish,
            subscribe,
            bindings,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct Operation {
    #[serde(rename = "operationId")]
    pub operation_id: String,
    pub message: OneOfMessages,
}

impl Operation {
    pub const fn new(operation_id: String, message: OneOfMessages) -> Self {
        Self {
            operation_id,
            message,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct OneOfMessages {
    #[serde(rename = "oneOf")]
    pub one_of: Vec<Message>,
}

impl OneOfMessages {
    pub fn new(one_of: Vec<Message>) -> Self {
        Self {
            one_of,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct Message {
    pub name: String,
    pub title: String,
    pub summary: String,
    pub description: String,
    #[serde(rename = "contentType")]
    pub content_type: String,
    pub payload: Schema,
    pub tags: Vec<Tag>,
}

impl Message {
    pub const fn new(
        name: String,
        title: String,
        summary: String,
        description: String,
        content_type: String,
        payload: Schema,
        tags: Vec<Tag>,
    ) -> Self {
        Self {
            name,
            title,
            summary,
            description,
            content_type,
            payload,
            tags,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct Components {
    pub schemas: BTreeMap<String, Schema>,
}

impl Components {
    pub fn new(schemas: BTreeMap<String, Schema>) -> Self {
        Self {
            schemas,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct ChannelBindings {
    pub ws: WebSocketsChannelBinding,
}

impl ChannelBindings {
    pub const fn new(ws: WebSocketsChannelBinding) -> Self {
        Self {
            ws,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct WebSocketsChannelBinding {
    #[serde(rename = "bindingVersion")]
    binding_version: String,
}

impl Default for WebSocketsChannelBinding {
    fn default() -> Self {
        Self {
            binding_version: String::from(WEB_SOCKET_CHANNEL_BINDINGS_VERSION),
        }
    }
}

#[derive(Debug, Serialize)]
pub struct Tag {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none", rename = "externalDocs")]
    pub external_docs: Option<ExternalDocumentation>,
}

impl Tag {
    pub const fn new(name: String, description: Option<String>, external_docs: Option<ExternalDocumentation>) -> Self {
        Self {
            name,
            description,
            external_docs,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct ExternalDocumentation {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub url: String,
}

impl ExternalDocumentation {
    pub const fn new(description: Option<String>, url: String) -> Self {
        Self {
            description,
            url,
        }
    }
}
