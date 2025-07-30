use crate::level_info::{Branch, Level};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Style, Modifier},
    text::{Span, Spans},
    widgets::{Block, Borders, Paragraph},
    Terminal,
};
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::io;


pub fn draw_cascade<B: Backend>(f: &mut Frame<B>, area: ratatui::layout::Rect, levels: &[Level], branches: &[Branch]) {
    let mut levels_sorted = levels.to_vec();
    levels_sorted.sort_by(|a, b| b.energy.partial_cmp(&a.energy).unwrap());

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(vec![Constraint::Length(3); levels_sorted.len()])
        .split(area);

    for (i, level) in levels_sorted.iter().enumerate() {
        let mut lines = vec![
            Spans::from(vec![
                Span::styled(format!("Level {}", level.idx), Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(format!(" — {:.3} ± {:.3} MeV", level.energy, level.denergy)),
            ]),
            Spans::from(vec![
                Span::raw(format!("Feeding: {:.3} ± {:.3}", level.feeding, level.dfeeding)),
            ]),
        ];

        // Add any transitions from this level
        for b in branches.iter().filter(|b| b.from == level.idx) {
            lines.push(Spans::from(Span::raw(format!(
                "↓ to {}: {:.3} ± {:.3}",
                b.to, b.val, b.dval
            ))));
        }

        let para = Paragraph::new(Text::from(lines))
            .block(Block::default().borders(Borders::ALL));
        f.render_widget(para, chunks[i]);
    }
}
