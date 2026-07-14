// SPDX-License-Identifier: GPL-3.0

use uuid::Uuid;

use crate::tag::Tag;

pub struct BoardContext<'a> {
    pub other_columns: &'a [(Uuid, String)],
    pub tags: &'a [Tag],
    pub search_query: &'a str,
}

impl BoardContext<'_> {
    pub fn matches(&self, title: &str, tag_ids: &[Uuid]) -> bool {
        if self.search_query.is_empty() {
            return true;
        }
        if title.to_lowercase().contains(self.search_query) {
            return true;
        }
        tag_ids.iter().any(|id| {
            self.tags
                .iter()
                .any(|t| t.id == *id && t.name.to_lowercase().contains(self.search_query))
        })
    }
}
