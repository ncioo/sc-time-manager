use ratatui::{
    layout::{Alignment, Rect},
    style::{Style, Stylize},
    text::{Line, Span},
    widgets::Paragraph,
    Frame,
};

use crate::{config::AfterExport, i18n::t, theme::theme};

#[derive(Clone, PartialEq)]
pub struct AfterExportChoice {
    pub mode: AfterExport,
    pub is_focused: bool,
}

impl AfterExportChoice {
    pub fn init(mode: AfterExport) -> AfterExportChoice {
        AfterExportChoice {
            mode,
            is_focused: false,
        }
    }

    pub fn set_focus(&mut self) {
        self.is_focused = true;
    }

    pub fn unset_focus(&mut self) {
        self.is_focused = false;
    }

    /// Следующий режим по кругу: ничего → открыть → скопировать → ничего.
    pub fn next(&mut self) {
        self.mode = match self.mode {
            AfterExport::Nothing => AfterExport::OpenFile,
            AfterExport::OpenFile => AfterExport::CopyList,
            AfterExport::CopyList => AfterExport::Nothing,
        };
    }

    /// Предыдущий режим по кругу.
    pub fn previous(&mut self) {
        self.mode = match self.mode {
            AfterExport::Nothing => AfterExport::CopyList,
            AfterExport::OpenFile => AfterExport::Nothing,
            AfterExport::CopyList => AfterExport::OpenFile,
        };
    }

    pub fn render(state: &AfterExportChoice, frame: &mut Frame, area: Rect) {
        let mut spans = vec![Span::styled(
            format!("{}: ", t("After closing", "После закрытия")),
            Style::new().fg(theme().muted),
        )];

        for (index, mode) in [
            AfterExport::Nothing,
            AfterExport::OpenFile,
            AfterExport::CopyList,
        ]
        .iter()
        .enumerate()
        {
            if index > 0 {
                spans.push(Span::raw("  "));
            }

            let style = if *mode == state.mode {
                if state.is_focused {
                    Style::new()
                        .fg(theme().accent_text)
                        .bg(theme().accent)
                        .bold()
                } else {
                    Style::new().fg(theme().accent).bold()
                }
            } else {
                Style::new().fg(theme().text)
            };

            spans.push(Span::styled(format!(" {} ", mode_label(*mode)), style));
        }

        if state.is_focused {
            spans.push(Span::styled(
                format!("  {}", t("←/→", "←/→")),
                Style::new().fg(theme().muted),
            ));
        }

        frame.render_widget(
            Paragraph::new(Line::from(spans)).alignment(Alignment::Center),
            area,
        );
    }
}

fn mode_label(mode: AfterExport) -> &'static str {
    match mode {
        AfterExport::Nothing => t("nothing", "ничего"),
        AfterExport::OpenFile => t("open .xlsx", "открыть .xlsx"),
        AfterExport::CopyList => t("copy list", "скопировать"),
    }
}
