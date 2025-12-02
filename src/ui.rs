use ratatui::{
    layout::{Alignment, Constraint, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame,
};

use crate::app::{App, AppState, CharResult, MenuField, TestMode, TIME_OPTIONS, WORD_OPTIONS};

const ASCII_TITLE: &[&str] = &[
    "████████╗██╗   ██╗██████╗ ███████╗████████╗██╗   ██╗██╗",
    "╚══██╔══╝╚██╗ ██╔╝██╔══██╗██╔════╝╚══██╔══╝██║   ██║██║",
    "   ██║    ╚████╔╝ ██████╔╝█████╗     ██║   ██║   ██║██║",
    "   ██║     ╚██╔╝  ██╔═══╝ ██╔══╝     ██║   ██║   ██║██║",
    "   ██║      ██║   ██║     ███████╗   ██║   ╚██████╔╝██║",
    "   ╚═╝      ╚═╝   ╚═╝     ╚══════╝   ╚═╝    ╚═════╝ ╚═╝",
];

pub fn draw(frame: &mut Frame, app: &App) {
    match app.state {
        AppState::Menu => draw_menu_screen(frame, app),
        AppState::Running => draw_typing_screen(frame, app),
        AppState::Finished => draw_results_screen(frame, app),
    }
}

fn draw_menu_screen(frame: &mut Frame, app: &App) {
    let area = frame.area();

    let chunks = Layout::vertical([
        Constraint::Length(8),  // ASCII art title
        Constraint::Length(1),  // Spacing
        Constraint::Length(9),  // Settings box
        Constraint::Min(1),     // Spacer
        Constraint::Length(2),  // Help text
    ])
    .split(area);

    // Draw ASCII title
    let title_lines: Vec<Line> = ASCII_TITLE
        .iter()
        .map(|line| {
            Line::from(Span::styled(
                *line,
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ))
        })
        .collect();

    let title = Paragraph::new(title_lines).alignment(Alignment::Center);
    frame.render_widget(title, chunks[0]);

    // Settings box - center it horizontally
    let settings_width = 40u16;
    let settings_x = area.width.saturating_sub(settings_width) / 2;
    let settings_area = Rect::new(
        settings_x,
        chunks[2].y,
        settings_width.min(area.width),
        chunks[2].height,
    );

    let settings_block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::DarkGray));
    frame.render_widget(settings_block, settings_area);

    // Inner area for settings content
    let inner_area = Rect::new(
        settings_area.x + 2,
        settings_area.y + 1,
        settings_area.width.saturating_sub(4),
        settings_area.height.saturating_sub(2),
    );

    let settings_rows = Layout::vertical([
        Constraint::Length(1),  // Spacer
        Constraint::Length(1),  // Mode
        Constraint::Length(1),  // Spacer
        Constraint::Length(1),  // Value
        Constraint::Length(1),  // Spacer
        Constraint::Length(1),  // Start
    ])
    .split(inner_area);

    // Mode selection
    let mode_selected = app.menu_field == MenuField::Mode;
    let mode_style = if mode_selected {
        Style::default()
            .fg(Color::Yellow)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::White)
    };

    let mode_text = match app.test_mode {
        TestMode::Time => "◄ Time  ►",
        TestMode::Words => "◄ Words ►",
    };

    let mode_line = Line::from(vec![
        Span::styled("    Mode:  ", Style::default().fg(Color::DarkGray)),
        Span::styled(mode_text, mode_style),
    ]);

    let mode_widget = Paragraph::new(mode_line);
    frame.render_widget(mode_widget, settings_rows[1]);

    // Value selection
    let value_selected = app.menu_field == MenuField::Value;
    let value_style = if value_selected {
        Style::default()
            .fg(Color::Yellow)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::White)
    };

    let (label, value_text) = match app.test_mode {
        TestMode::Time => {
            let secs = app.time_seconds();
            let display = if secs >= 60 {
                format!("{}m {}s", secs / 60, secs % 60)
            } else {
                format!("{}s", secs)
            };
            // Show arrows only if not at boundary
            let left = if app.time_option_idx > 0 { "◄ " } else { "  " };
            let right = if app.time_option_idx < TIME_OPTIONS.len() - 1 {
                " ►"
            } else {
                "  "
            };
            ("    Time:", format!("{}{:^7}{}", left, display, right))
        }
        TestMode::Words => {
            let count = app.target_word_count();
            let left = if app.word_option_idx > 0 { "◄ " } else { "  " };
            let right = if app.word_option_idx < WORD_OPTIONS.len() - 1 {
                " ►"
            } else {
                "  "
            };
            ("   Words:", format!("{}{:^7}{}", left, count, right))
        }
    };

    let value_line = Line::from(vec![
        Span::styled(format!("{}  ", label), Style::default().fg(Color::DarkGray)),
        Span::styled(value_text, value_style),
    ]);

    let value_widget = Paragraph::new(value_line);
    frame.render_widget(value_widget, settings_rows[3]);

    // Start button
    let start_selected = app.menu_field == MenuField::Start;
    let start_style = if start_selected {
        Style::default()
            .fg(Color::Black)
            .bg(Color::Green)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::Green)
    };

    let start_line = Line::from(vec![
        Span::raw("         "),
        Span::styled("[ Start ]", start_style),
    ]);
    let start_widget = Paragraph::new(start_line);
    frame.render_widget(start_widget, settings_rows[5]);

    // Help text
    let help_line = Line::from(Span::styled(
        "↑↓/jk: navigate  ←→/hl: change  Enter: select  q: quit",
        Style::default().fg(Color::DarkGray),
    ));
    let help_widget = Paragraph::new(help_line).alignment(Alignment::Center);
    frame.render_widget(help_widget, chunks[4]);
}

fn draw_typing_screen(frame: &mut Frame, app: &App) {
    let area = frame.area();

    let chunks = Layout::vertical([
        Constraint::Length(3),
        Constraint::Min(5),
        Constraint::Length(3),
    ])
    .split(area);

    draw_timer(frame, app, chunks[0]);
    draw_words(frame, app, chunks[1]);
    draw_input(frame, app, chunks[2]);
}

fn draw_timer(frame: &mut Frame, app: &App, area: Rect) {
    let timer_text = if app.start_time.is_none() {
        "Type to start...".to_string()
    } else {
        match app.test_mode {
            TestMode::Time => {
                let remaining = app.time_remaining();
                format!("{:.1}s", remaining.as_secs_f64())
            }
            TestMode::Words => {
                let elapsed = app.time_elapsed();
                let progress = format!(
                    "{}/{} words  |  {:.1}s",
                    app.current_word_idx,
                    app.words.len(),
                    elapsed.as_secs_f64()
                );
                progress
            }
        }
    };

    let color = match app.test_mode {
        TestMode::Time => {
            let secs = app.time_remaining().as_secs_f64();
            if secs <= 5.0 && app.start_time.is_some() {
                Color::Red
            } else if secs <= 10.0 && app.start_time.is_some() {
                Color::Yellow
            } else {
                Color::Green
            }
        }
        TestMode::Words => Color::Cyan,
    };

    let title = match app.test_mode {
        TestMode::Time => "Timer",
        TestMode::Words => "Progress",
    };

    let timer = Paragraph::new(timer_text)
        .style(Style::default().fg(color).add_modifier(Modifier::BOLD))
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL).title(title));

    frame.render_widget(timer, area);
}

fn draw_words(frame: &mut Frame, app: &App, area: Rect) {
    let mut spans: Vec<Span> = Vec::new();

    for (idx, word) in app.words.iter().enumerate() {
        if idx < app.current_word_idx {
            let style = if app.word_completed_correctly(idx) {
                Style::default().fg(Color::Green)
            } else {
                Style::default().fg(Color::Red)
            };
            spans.push(Span::styled(word.clone(), style));
            spans.push(Span::raw(" "));
        } else if idx == app.current_word_idx {
            for (char_idx, c) in word.chars().enumerate() {
                let style = if char_idx < app.typed_input.len() {
                    let result = app.word_char_results[idx].get(char_idx);
                    match result {
                        Some(CharResult::Correct) => Style::default().fg(Color::Green),
                        Some(CharResult::Incorrect) => {
                            Style::default().fg(Color::White).bg(Color::Red)
                        }
                        None => Style::default().fg(Color::DarkGray),
                    }
                } else if char_idx == app.typed_input.len() {
                    Style::default()
                        .fg(Color::White)
                        .add_modifier(Modifier::UNDERLINED)
                } else {
                    Style::default().fg(Color::DarkGray)
                };
                spans.push(Span::styled(c.to_string(), style));
            }

            if app.typed_input.len() > word.len() {
                let extra: String = app.typed_input.chars().skip(word.len()).collect();
                spans.push(Span::styled(
                    extra,
                    Style::default().fg(Color::White).bg(Color::Red),
                ));
            }

            spans.push(Span::raw(" "));
        } else {
            spans.push(Span::styled(
                word.clone(),
                Style::default().fg(Color::DarkGray),
            ));
            spans.push(Span::raw(" "));
        }
    }

    let line = Line::from(spans);
    let words_paragraph = Paragraph::new(line)
        .wrap(Wrap { trim: false })
        .block(Block::default().borders(Borders::ALL).title("Words"));

    frame.render_widget(words_paragraph, area);
}

fn draw_input(frame: &mut Frame, app: &App, area: Rect) {
    let input_text = format!("> {}_", app.typed_input);
    let input = Paragraph::new(input_text)
        .style(Style::default().fg(Color::White))
        .block(Block::default().borders(Borders::ALL).title("Input"));

    frame.render_widget(input, area);
}

fn draw_results_screen(frame: &mut Frame, app: &App) {
    let area = frame.area();
    let stats = app.calculate_stats();

    let text = vec![
        Line::from(""),
        Line::from(vec![
            Span::raw("WPM: "),
            Span::styled(
                format!("{:.0}", stats.wpm),
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::raw("Accuracy: "),
            Span::styled(
                format!("{:.1}%", stats.accuracy),
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::raw("Total Errors: "),
            Span::styled(
                format!("{}", stats.total_errors),
                Style::default()
                    .fg(Color::Red)
                    .add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::raw("Words with mistakes: "),
            Span::styled(
                format!("{}", stats.words_with_errors),
                Style::default()
                    .fg(Color::Red)
                    .add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::raw("Characters: "),
            Span::styled(
                format!("{}", stats.correct_chars),
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw("/"),
            Span::styled(
                format!("{}", stats.total_chars_typed),
                Style::default().add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(""),
        Line::from(""),
        Line::from(Span::styled(
            "Press 'r' to return to menu | Press 'q' to quit",
            Style::default().fg(Color::DarkGray),
        )),
    ];

    let results = Paragraph::new(text)
        .alignment(Alignment::Center)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Results")
                .title_alignment(Alignment::Center),
        );

    frame.render_widget(results, area);
}
