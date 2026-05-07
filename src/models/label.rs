// SPDX-License-Identifier: GPL-3.0

use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Label {
    pub id: Uuid,
    pub name: String,
    pub color: LabelColor,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum LabelColor {
    Red,
    Orange,
    Yellow,
    Green,
    Blue,
    Purple,
    Pink,
    Teal,
    Gray,
}

impl LabelColor {
    /// Returns the COSMIC iced Color for this label
    pub fn as_color(&self) -> cosmic::iced::Color {
        match self {
            LabelColor::Red => cosmic::iced::Color::from_rgb8(0xef, 0x44, 0x44),
            LabelColor::Orange => cosmic::iced::Color::from_rgb8(0xf9, 0x73, 0x16),
            LabelColor::Yellow => cosmic::iced::Color::from_rgb8(0xea, 0xb3, 0x08),
            LabelColor::Green => cosmic::iced::Color::from_rgb8(0x22, 0xc5, 0x5e),
            LabelColor::Blue => cosmic::iced::Color::from_rgb8(0x3b, 0x82, 0xf6),
            LabelColor::Purple => cosmic::iced::Color::from_rgb8(0xa8, 0x55, 0xf7),
            LabelColor::Pink => cosmic::iced::Color::from_rgb8(0xec, 0x48, 0x99),
            LabelColor::Teal => cosmic::iced::Color::from_rgb8(0x14, 0xb8, 0xa6),
            LabelColor::Gray => cosmic::iced::Color::from_rgb8(0x6b, 0x72, 0x80),
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            LabelColor::Red => "Red",
            LabelColor::Orange => "Orange",
            LabelColor::Yellow => "Yellow",
            LabelColor::Green => "Green",
            LabelColor::Blue => "Blue",
            LabelColor::Purple => "Purple",
            LabelColor::Pink => "Pink",
            LabelColor::Teal => "Teal",
            LabelColor::Gray => "Gray",
        }
    }
}
