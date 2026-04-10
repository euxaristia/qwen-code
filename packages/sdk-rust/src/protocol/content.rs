//! Content domain types (content blocks for prompts).

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "kebab-case")]
pub enum ContentBlock {
    Text(TextContent),
    Image(ImageContent),
    Audio(AudioContent),
    #[serde(rename = "resource_link")]
    ResourceLink(ResourceLink),
}

impl ContentBlock {
    pub fn text(text: impl Into<String>) -> Self {
        Self::Text(TextContent { text: text.into() })
    }
}

impl From<String> for ContentBlock {
    fn from(text: String) -> Self {
        Self::text(text)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextContent {
    pub text: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageContent {
    pub data: String,
    pub mime_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioContent {
    pub data: String,
    pub mime_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceLink {
    pub uri: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mime_type: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Annotations {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub audience: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub priority: Option<f64>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_content_block_text() {
        let block = ContentBlock::text("Hello, world!");
        match block {
            ContentBlock::Text(TextContent { text }) => {
                assert_eq!(text, "Hello, world!");
            }
            _ => panic!("Expected Text content block"),
        }
    }

    #[test]
    fn test_content_block_from_string() {
        let block: ContentBlock = "test message".to_string().into();
        match block {
            ContentBlock::Text(TextContent { text }) => {
                assert_eq!(text, "test message");
            }
            _ => panic!("Expected Text content block"),
        }
    }

    #[test]
    fn test_content_block_image_serialization() {
        let block = ContentBlock::Image(ImageContent {
            data: "base64data".to_string(),
            mime_type: "image/png".to_string(),
        });
        let json = serde_json::to_string(&block).unwrap();
        assert!(json.contains("\"type\":\"image\""));
        assert!(json.contains("\"mime_type\":\"image/png\""));
    }

    #[test]
    fn test_content_block_image_deserialization() {
        let json = r#"{"type":"image","data":"abc123","mime_type":"image/jpeg"}"#;
        let block: ContentBlock = serde_json::from_str(json).unwrap();
        match block {
            ContentBlock::Image(img) => {
                assert_eq!(img.data, "abc123");
                assert_eq!(img.mime_type, "image/jpeg");
            }
            _ => panic!("Expected Image content block"),
        }
    }

    #[test]
    fn test_content_block_audio_serialization() {
        let block = ContentBlock::Audio(AudioContent {
            data: "audio_base64".to_string(),
            mime_type: "audio/wav".to_string(),
        });
        let json = serde_json::to_string(&block).unwrap();
        assert!(json.contains("\"type\":\"audio\""));
        assert!(json.contains("\"mime_type\":\"audio/wav\""));
    }

    #[test]
    fn test_content_block_audio_deserialization() {
        let json = r#"{"type":"audio","data":"audiodata","mime_type":"audio/mp3"}"#;
        let block: ContentBlock = serde_json::from_str(json).unwrap();
        match block {
            ContentBlock::Audio(audio) => {
                assert_eq!(audio.data, "audiodata");
                assert_eq!(audio.mime_type, "audio/mp3");
            }
            _ => panic!("Expected Audio content block"),
        }
    }

    #[test]
    fn test_content_block_resource_link_serialization() {
        let block = ContentBlock::ResourceLink(ResourceLink {
            uri: "file:///path/to/file.txt".to_string(),
            mime_type: Some("text/plain".to_string()),
        });
        let json = serde_json::to_string(&block).unwrap();
        assert!(json.contains("\"type\":\"resource_link\""));
        assert!(json.contains("\"uri\":\"file:///path/to/file.txt\""));
    }

    #[test]
    fn test_content_block_resource_link_deserialization() {
        let json = r#"{"type":"resource_link","uri":"https://example.com/doc","mime_type":null}"#;
        let block: ContentBlock = serde_json::from_str(json).unwrap();
        match block {
            ContentBlock::ResourceLink(link) => {
                assert_eq!(link.uri, "https://example.com/doc");
            }
            _ => panic!("Expected ResourceLink content block"),
        }
    }

    #[test]
    fn test_content_block_resource_link_no_mime_type() {
        let block = ContentBlock::ResourceLink(ResourceLink {
            uri: "file:///no-mime".to_string(),
            mime_type: None,
        });
        let json = serde_json::to_string(&block).unwrap();
        assert!(json.contains("\"uri\":\"file:///no-mime\""));
        // mime_type should be skipped when None
        assert!(!json.contains("\"mime_type\""));
    }

    #[test]
    fn test_content_block_text_serialization() {
        let block = ContentBlock::text("hello");
        let json = serde_json::to_string(&block).unwrap();
        assert!(json.contains("\"type\":\"text\""));
        assert!(json.contains("\"text\":\"hello\""));
    }

    #[test]
    fn test_content_block_text_deserialization() {
        let json = r#"{"type":"text","text":"deserialized"}"#;
        let block: ContentBlock = serde_json::from_str(json).unwrap();
        match block {
            ContentBlock::Text(tc) => assert_eq!(tc.text, "deserialized"),
            _ => panic!("Expected Text content block"),
        }
    }

    #[test]
    fn test_annotations_serialization() {
        let annotations = Annotations {
            audience: Some(vec!["assistant".to_string(), "user".to_string()]),
            priority: Some(0.8),
        };
        let json = serde_json::to_string(&annotations).unwrap();
        assert!(json.contains("\"audience\""));
        assert!(json.contains("\"priority\":0.8"));
    }

    #[test]
    fn test_annotations_skip_none() {
        let annotations = Annotations {
            audience: None,
            priority: None,
        };
        let json = serde_json::to_string(&annotations).unwrap();
        assert_eq!(json, "{}");
    }

    #[test]
    fn test_annotations_deserialization() {
        let json = r#"{"audience":["model"],"priority":0.5}"#;
        let annotations: Annotations = serde_json::from_str(json).unwrap();
        assert_eq!(annotations.audience, Some(vec!["model".to_string()]));
        assert_eq!(annotations.priority, Some(0.5));
    }

    #[test]
    fn test_annotations_partial() {
        let json = r#"{"priority":1.0}"#;
        let annotations: Annotations = serde_json::from_str(json).unwrap();
        assert!(annotations.audience.is_none());
        assert_eq!(annotations.priority, Some(1.0));
    }
}
