// SPDX-License-Identifier: GPL-3.0

pub mod message;

use std::borrow::Cow;

use cosmic::iced::clipboard::mime::{AllowedMimeTypes, AsMimeTypes};
use uuid::Uuid;

pub const CARD_MIME: &str = "application/x-boards-card";

#[derive(Debug, Clone)]
pub struct CardDragData {
    pub card_id: Uuid,
}

impl AsMimeTypes for CardDragData {
    fn available(&self) -> Cow<'static, [String]> {
        Cow::Owned(vec![CARD_MIME.to_string()])
    }

    fn as_bytes(&self, mime_type: &str) -> Option<Cow<'static, [u8]>> {
        if mime_type == CARD_MIME {
            Some(Cow::Owned(self.card_id.to_string().into_bytes()))
        } else {
            None
        }
    }
}

impl AllowedMimeTypes for CardDragData {
    fn allowed() -> Cow<'static, [String]> {
        Cow::Owned(vec![CARD_MIME.to_string()])
    }
}

impl TryFrom<(Vec<u8>, String)> for CardDragData {
    type Error = String;

    fn try_from((data, mime): (Vec<u8>, String)) -> Result<Self, Self::Error> {
        if mime != CARD_MIME {
            return Err(format!("unexpected MIME type: {mime}"));
        }
        let text = String::from_utf8(data).map_err(|e| e.to_string())?;
        let card_id = Uuid::parse_str(text.trim()).map_err(|e| e.to_string())?;
        Ok(CardDragData { card_id })
    }
}
