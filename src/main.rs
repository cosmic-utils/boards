// SPDX-License-Identifier: GPL-3.0

mod app;
mod board;
mod card;
mod checklist;
mod column;
mod config;
mod dialog;
mod dnd;
mod i18n;
mod menu_bar;
mod storage;
mod tag;
mod widgets;

fn main() -> cosmic::iced::Result {
    let requested_languages = i18n_embed::DesktopLanguageRequester::requested_languages();

    i18n::init(&requested_languages);

    let settings = cosmic::app::Settings::default().size_limits(
        cosmic::iced::Limits::NONE
            .min_width(360.0)
            .min_height(180.0),
    );

    cosmic::app::run::<app::AppModel>(settings, ())
}
