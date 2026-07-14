// SPDX-License-Identifier: GPL-3.0

use cosmic::cosmic_theme::Spacing;
use cosmic::iced::{Alignment, Length};
use cosmic::prelude::*;
use cosmic::widget;
use uuid::Uuid;

use crate::app::{AppModel, DialogPage, Message};
use crate::board::Board;
use crate::board::context::BoardContext;
use crate::board::dialog::NewBoardDialog;
use crate::fl;
use crate::list::dialog::NewListDialog;
use crate::list::widget::{view_list, view_list_with_input};
use crate::widgets::content_unavailable::content_unavailable;

impl AppModel {
    pub fn view_empty_state(&self) -> Element<'_, Message> {
        let space_m = cosmic::theme::spacing().space_m;

        let content = widget::column::with_capacity(3)
            .push(widget::text::title2(fl!("no-boards")))
            .push(widget::Space::new().height(space_m))
            .push(
                widget::button::suggested(fl!("create-first-board")).on_press(
                    Message::OpenDialogPage(DialogPage::NewBoard(NewBoardDialog::new())),
                ),
            )
            .align_x(Alignment::Center)
            .spacing(space_m);

        widget::container(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .align_x(cosmic::iced::alignment::Horizontal::Center)
            .align_y(cosmic::iced::alignment::Vertical::Center)
            .into()
    }

    pub fn view_board<'a>(&'a self, board: &'a Board) -> Element<'a, Message> {
        let Spacing {
            space_m, space_l, ..
        } = cosmic::theme::spacing();

        if board.lists.is_empty() {
            return content_unavailable(fl!("no-lists"))
                .description(fl!("add-list-description"))
                .action(
                    fl!("add-list"),
                    Message::OpenDialogPage(DialogPage::NewList(NewListDialog::new())),
                )
                .into();
        }

        let mut row = widget::row::with_capacity(board.lists.len() + 1)
            .spacing(space_m)
            .padding([space_m, space_l])
            .height(Length::Fill);

        let search_query = self.search_query.to_lowercase();

        for list in &board.lists {
            let (drag_hovered_card_id, tail_active) = match self.drag_hover {
                Some((lid, Some(cid))) if lid == list.id => (Some(cid), false),
                Some((lid, None)) if lid == list.id => (None, true),
                _ => (None, false),
            };
            let editing_title = self.editing_list_title == Some(list.id);
            let other_lists: Vec<(Uuid, String)> = board
                .lists
                .iter()
                .filter(|l| l.id != list.id)
                .map(|l| (l.id, l.title.clone()))
                .collect();
            let ctx = BoardContext {
                other_lists: &other_lists,
                tags: &board.tags,
                search_query: &search_query,
            };
            let col = if let Some((ref input_id, ref active_list_id, ref input_text)) =
                self.new_card_input
            {
                if *active_list_id == list.id {
                    view_list_with_input(
                        list,
                        editing_title,
                        &ctx,
                        input_text,
                        input_id,
                        drag_hovered_card_id,
                        tail_active,
                    )
                } else {
                    view_list(list, editing_title, &ctx, drag_hovered_card_id, tail_active)
                }
            } else {
                view_list(list, editing_title, &ctx, drag_hovered_card_id, tail_active)
            };
            row = row.push(col);
        }

        widget::scrollable::horizontal(row)
            .height(Length::Fill)
            .into()
    }
}
