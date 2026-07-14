use cosmic::iced::Alignment;
use cosmic::theme::spacing;
use cosmic::{Element, widget};

pub struct ContentUnavailable<Message: 'static + Clone> {
    title: String,
    description: Option<String>,
    action: Option<(String, Message)>,
    icon: Option<String>,
}

pub fn content_unavailable<'a, Message: 'static + Clone>(
    title: impl Into<String>,
) -> ContentUnavailable<Message> {
    ContentUnavailable::new(title)
}

impl<Message: 'static + Clone> ContentUnavailable<Message> {
    #[allow(unused)]
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            description: None,
            action: None,
            icon: None,
        }
    }

    #[allow(unused)]
    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title = title.into();
        self
    }

    #[allow(unused)]
    pub fn icon(mut self, icon: impl Into<String>) -> Self {
        self.icon = Some(icon.into());
        self
    }

    #[allow(unused)]
    pub fn description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    #[allow(unused)]
    pub fn action(mut self, label: impl Into<String>, msg: Message) -> Self {
        self.action = Some((label.into(), msg));
        self
    }

    #[allow(unused)]
    fn build<'a>(self) -> Element<'a, Message> {
        let mut content: Vec<Element<'a, Message>> = vec![];

        if let Some(icon) = self.icon {
            let icon = widget::icon::from_name(icon).size(48);
            content.push(icon.into());
        }

        content.push(
            widget::text::heading(self.title.to_string())
                .size(24)
                .into(),
        );

        if let Some(desc) = self.description {
            content.push(widget::text(desc.to_string()).size(16).into());
        }

        if let Some((label, msg)) = self.action {
            content.push(
                widget::button::suggested(label.to_string())
                    .on_press(msg)
                    .into(),
            );
        }

        cosmic::iced::widget::center(
            widget::column::with_children(content)
                .spacing(spacing().space_s)
                .align_x(Alignment::Center)
                .max_width(300),
        )
        .into()
    }
}

impl<'a, Message: 'static + Clone> Into<Element<'a, Message>> for ContentUnavailable<Message> {
    fn into(self) -> Element<'a, Message> {
        self.build()
    }
}
