// SPDX-License-Identifier: GPL-3.0

use cosmic::iced::Length;
use cosmic::prelude::*;
use cosmic::widget;

use crate::app::Message;
use crate::models::board::Board;
use crate::widgets::list_column::view_list;

pub fn view_board<'a>(board: &'a Board) -> Element<'a, Message> {
    let space_m = cosmic::theme::spacing().space_m;
    let space_l = cosmic::theme::spacing().space_l;

    let columns = board.lists.iter().map(|list| view_list(list, None, false));

    let board_row = widget::row::with_capacity(board.lists.len())
        .extend(columns)
        .spacing(space_m)
        .padding([space_m, space_l])
        .height(Length::Fill);

    widget::scrollable::horizontal(board_row)
        .height(Length::Fill)
        .into()
}
