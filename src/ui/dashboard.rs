use anyhow::Result;
use crossterm::{
    event::{self, Event, KeyCode, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Cell, Paragraph, Row, Table, TableState},
    Frame, Terminal,
};
use std::io;
use crate::models::Setup;

pub struct Dashboard {
    setups: Vec<Setup>,
    table_state: TableState,
    selected_idx: usize,
}

impl Dashboard {
    pub fn new(setups: Vec<Setup>) -> Self {
        let mut table_state = TableState::default();
        table_state.select(Some(0));
        Self { setups, table_state, selected_idx: 0 }
    }

    pub fn run(mut self) -> Result<()> {
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen)?;
        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend)?;

        loop {
            terminal.draw(|f| self.render(f))?;

            if event::poll(std::time::Duration::from_millis(200))? {
                if let Event::Key(key) = event::read()? {
                    match (key.code, key.modifiers) {
                        (KeyCode::Char('q'), _)
                        | (KeyCode::Char('c'), KeyModifiers::CONTROL) => break,
                        (KeyCode::Down | KeyCode::Char('j'), _) => self.next(),
                        (KeyCode::Up | KeyCode::Char('k'), _) => self.prev(),
                        _ => {}
                    }
                }
            }
        }

        disable_raw_mode()?;
        execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
        Ok(())
    }

    fn next(&mut self) {
        if !self.setups.is_empty() {
            self.selected_idx = (self.selected_idx + 1) % self.setups.len();
            self.table_state.select(Some(self.selected_idx));
        }
    }

    fn prev(&mut self) {
        if !self.setups.is_empty() {
            self.selected_idx = self.selected_idx.saturating_sub(1);
            self.table_state.select(Some(self.selected_idx));
        }
    }

    fn render(&mut self, f: &mut Frame) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),
                Constraint::Min(10),
                Constraint::Length(10),
                Constraint::Length(2),
            ])
            .split(f.area());

        self.render_header(f, chunks[0]);
        self.render_table(f, chunks[1]);
        self.render_detail(f, chunks[2]);
        self.render_footer(f, chunks[3]);
    }

    fn render_header(&self, f: &mut Frame, area: Rect) {
        let title = Paragraph::new(Line::from(vec![
            Span::styled("SETUPS", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
            Span::raw("  |  Pre-Market Setup Scanner  |  "),
            Span::styled(
                format!("{} setups found", self.setups.len()),
                Style::default().fg(Color::Yellow),
            ),
        ]))
        .block(Block::default().borders(Borders::ALL));
        f.render_widget(title, area);
    }

    fn render_table(&mut self, f: &mut Frame, area: Rect) {
        let header = Row::new(vec![
            Cell::from("Symbol").style(Style::default().add_modifier(Modifier::BOLD)),
            Cell::from("Price").style(Style::default().add_modifier(Modifier::BOLD)),
            Cell::from("Gap %").style(Style::default().add_modifier(Modifier::BOLD)),
            Cell::from("RVOL").style(Style::default().add_modifier(Modifier::BOLD)),
            Cell::from("Float").style(Style::default().add_modifier(Modifier::BOLD)),
            Cell::from("Short %").style(Style::default().add_modifier(Modifier::BOLD)),
            Cell::from("Catalyst").style(Style::default().add_modifier(Modifier::BOLD)),
            Cell::from("Score").style(Style::default().add_modifier(Modifier::BOLD)),
            Cell::from("Grade").style(Style::default().add_modifier(Modifier::BOLD)),
        ])
        .style(Style::default().fg(Color::White))
        .height(1);

        let rows: Vec<Row> = self
            .setups
            .iter()
            .map(|s| {
                let gap = s.ticker.gap_pct();
                let gap_color = if gap > 0.0 { Color::Green } else { Color::Red };
                let rvol = s.ticker.relative_volume();
                let float_str = match s.ticker.float_shares {
                    Some(f) if f < 1_000_000 => format!("{:.0}K", f as f64 / 1000.0),
                    Some(f) if f < 1_000_000_000 => format!("{:.1}M", f as f64 / 1_000_000.0),
                    Some(f) => format!("{:.1}B", f as f64 / 1_000_000_000.0),
                    None => "N/A".to_string(),
                };
                Row::new(vec![
                    Cell::from(s.ticker.symbol.clone())
                        .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
                    Cell::from(format!("${:.2}", s.ticker.premarket_price)),
                    Cell::from(format!("{:+.1}%", gap))
                        .style(Style::default().fg(gap_color)),
                    Cell::from(format!("{:.1}x", rvol))
                        .style(if rvol >= 3.0 {
                            Style::default().fg(Color::Yellow)
                        } else {
                            Style::default()
                        }),
                    Cell::from(float_str),
                    Cell::from(
                        s.ticker
                            .short_float_pct
                            .map(|p| format!("{:.1}%", p))
                            .unwrap_or_else(|| "N/A".to_string()),
                    ),
                    Cell::from(s.catalyst.to_string()),
                    Cell::from(format!("{:.0}", s.score.total))
                        .style(score_color(s.score.total)),
                    Cell::from(s.score.grade()).style(score_color(s.score.total)),
                ])
            })
            .collect();

        let table = Table::new(
            rows,
            [
                Constraint::Length(8),
                Constraint::Length(9),
                Constraint::Length(8),
                Constraint::Length(7),
                Constraint::Length(8),
                Constraint::Length(9),
                Constraint::Min(16),
                Constraint::Length(7),
                Constraint::Length(6),
            ],
        )
        .header(header)
        .block(Block::default().borders(Borders::ALL).title(" Top Setups "))
        .row_highlight_style(Style::default().bg(Color::DarkGray).add_modifier(Modifier::BOLD));

        f.render_stateful_widget(table, area, &mut self.table_state);
    }

    fn render_detail(&self, f: &mut Frame, area: Rect) {
        let content = if let Some(setup) = self.setups.get(self.selected_idx) {
            let headline = setup
                .catalyst_headline
                .as_deref()
                .unwrap_or("No news available");
            let calls = setup
                .unusual_options_calls
                .map(|c| format!("${:.0}K", c / 1000.0))
                .unwrap_or_else(|| "N/A".to_string());
            let puts = setup
                .unusual_options_puts
                .map(|p| format!("${:.0}K", p / 1000.0))
                .unwrap_or_else(|| "N/A".to_string());
            vec![
                Line::from(vec![
                    Span::styled(
                        format!("  {} ", setup.ticker.symbol),
                        Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
                    ),
                    Span::raw(format!(
                        "| Direction: {} | Score Breakdown: Gap {:.0}  Vol {:.0}  Catalyst {:.0}  Squeeze {:.0}  Options {:.0}",
                        setup.direction(),
                        setup.score.gap_score,
                        setup.score.volume_score,
                        setup.score.catalyst_score,
                        setup.score.squeeze_score,
                        setup.score.options_score,
                    )),
                ]),
                Line::from(vec![
                    Span::raw(format!("  News: {}", headline)),
                ]),
                Line::from(vec![
                    Span::raw(format!(
                        "  Options Flow — Calls: {}  Puts: {}  | Prev Close: ${:.2}",
                        calls, puts, setup.ticker.prev_close
                    )),
                ]),
            ]
        } else {
            vec![Line::from("  Select a setup to view details")]
        };

        let detail = Paragraph::new(content)
            .block(Block::default().borders(Borders::ALL).title(" Detail "));
        f.render_widget(detail, area);
    }

    fn render_footer(&self, f: &mut Frame, area: Rect) {
        let footer = Paragraph::new(Line::from(vec![
            Span::styled(" ↑/k ", Style::default().fg(Color::Yellow)),
            Span::raw("up  "),
            Span::styled("↓/j ", Style::default().fg(Color::Yellow)),
            Span::raw("down  "),
            Span::styled("q ", Style::default().fg(Color::Yellow)),
            Span::raw("quit"),
        ]));
        f.render_widget(footer, area);
    }
}

fn score_color(score: f64) -> Style {
    match score as u32 {
        90..=100 => Style::default().fg(Color::LightGreen).add_modifier(Modifier::BOLD),
        70..=89 => Style::default().fg(Color::Green),
        50..=69 => Style::default().fg(Color::Yellow),
        _ => Style::default().fg(Color::Red),
    }
}
