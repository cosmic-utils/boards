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
use crate::column::dialog::NewColumnDialog;
use crate::column::widget::{view_column, view_column_with_input};
use crate::fl;
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

        if board.columns.is_empty() {
            return content_unavailable(fl!("no-columns"))
                .description(fl!("add-column-description"))
                .action(
                    fl!("add-column"),
                    Message::OpenDialogPage(DialogPage::NewColumn(NewColumnDialog::new())),
                )
                .into();
        }

        let mut row = widget::row::with_capacity(board.columns.len() + 1)
            .spacing(space_m)
            .padding([space_m, space_l])
            .height(Length::Fill);

        let search_query = self.search_query.to_lowercase();

        for column in &board.columns {
            let (drag_hovered_card_id, tail_active) = match self.drag_hover {
                Some((lid, Some(cid))) if lid == column.id => (Some(cid), false),
                Some((lid, None)) if lid == column.id => (None, true),
                _ => (None, false),
            };
            let editing_title = self.editing_column_title == Some(column.id);
            let other_columns: Vec<(Uuid, String)> = board
                .columns
                .iter()
                .filter(|l| l.id != column.id)
                .map(|l| (l.id, l.title.clone()))
                .collect();
            let ctx = BoardContext {
                other_columns: &other_columns,
                tags: &board.tags,
                search_query: &search_query,
            };
            let col = if let Some((ref input_id, ref active_column_id, ref input_text)) =
                self.new_card_input
            {
                if *active_column_id == column.id {
                    view_column_with_input(
                        column,
                        editing_title,
                        &ctx,
                        input_text,
                        input_id,
                        drag_hovered_card_id,
                        tail_active,
                    )
                } else {
                    view_column(
                        column,
                        editing_title,
                        &ctx,
                        drag_hovered_card_id,
                        tail_active,
                    )
                }
            } else {
                view_column(
                    column,
                    editing_title,
                    &ctx,
                    drag_hovered_card_id,
                    tail_active,
                )
            };
            row = row.push(col);
        }

        widget::scrollable::horizontal(row)
            .height(Length::Fill)
            .into()
    }
}
