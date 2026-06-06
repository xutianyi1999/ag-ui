use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;

/// Source for multimodal input content — inline data or URL.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum InputContentSource {
    /// Base64-encoded inline data.
    Data {
        value: String,
        #[serde(rename = "mimeType")]
        mime_type: String,
    },
    /// Remote URL.
    Url {
        value: String,
        #[serde(rename = "mimeType", skip_serializing_if = "Option::is_none")]
        mime_type: Option<String>,
    },
}

/// Text content part for multimodal user messages.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TextInputContent {
    #[serde(rename = "type")]
    pub content_type: String,
    pub text: String,
}

impl TextInputContent {
    pub fn new(text: String) -> Self {
        Self {
            content_type: "text".to_string(),
            text,
        }
    }
}

/// Image content part.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ImageInputContent {
    #[serde(rename = "type")]
    pub content_type: String,
    pub source: InputContentSource,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<JsonValue>,
}

impl ImageInputContent {
    pub fn new(source: InputContentSource) -> Self {
        Self {
            content_type: "image".to_string(),
            source,
            metadata: None,
        }
    }
}

/// Audio content part.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AudioInputContent {
    #[serde(rename = "type")]
    pub content_type: String,
    pub source: InputContentSource,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<JsonValue>,
}

impl AudioInputContent {
    pub fn new(source: InputContentSource) -> Self {
        Self {
            content_type: "audio".to_string(),
            source,
            metadata: None,
        }
    }
}

/// Video content part.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VideoInputContent {
    #[serde(rename = "type")]
    pub content_type: String,
    pub source: InputContentSource,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<JsonValue>,
}

impl VideoInputContent {
    pub fn new(source: InputContentSource) -> Self {
        Self {
            content_type: "video".to_string(),
            source,
            metadata: None,
        }
    }
}

/// Document content part.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DocumentInputContent {
    #[serde(rename = "type")]
    pub content_type: String,
    pub source: InputContentSource,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<JsonValue>,
}

impl DocumentInputContent {
    pub fn new(source: InputContentSource) -> Self {
        Self {
            content_type: "document".to_string(),
            source,
            metadata: None,
        }
    }
}

/// Legacy binary input content.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BinaryInputContent {
    #[serde(rename = "type")]
    pub content_type: String,
    #[serde(rename = "mimeType")]
    pub mime_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub filename: Option<String>,
}

impl BinaryInputContent {
    pub fn new(mime_type: String) -> Self {
        Self {
            content_type: "binary".to_string(),
            mime_type,
            id: None,
            url: None,
            data: None,
            filename: None,
        }
    }
}

/// Multimodal input content part — discriminated by `type`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum InputContent {
    /// Plain text.
    Text {
        text: String,
    },
    /// Image (base64 data or URL).
    Image {
        source: InputContentSource,
        #[serde(skip_serializing_if = "Option::is_none")]
        metadata: Option<JsonValue>,
    },
    /// Audio (base64 data or URL).
    Audio {
        source: InputContentSource,
        #[serde(skip_serializing_if = "Option::is_none")]
        metadata: Option<JsonValue>,
    },
    /// Video (base64 data or URL).
    Video {
        source: InputContentSource,
        #[serde(skip_serializing_if = "Option::is_none")]
        metadata: Option<JsonValue>,
    },
    /// Document (base64 data or URL).
    Document {
        source: InputContentSource,
        #[serde(skip_serializing_if = "Option::is_none")]
        metadata: Option<JsonValue>,
    },
    /// Legacy binary format.
    Binary {
        #[serde(rename = "mimeType")]
        mime_type: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        id: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        url: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        data: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        filename: Option<String>,
    },
}

/// MessageContent is either a plain string or an array of InputContent parts.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum MessageContent {
    /// Simple text content.
    Text(String),
    /// Multimodal content parts.
    Parts(Vec<InputContent>),
}
