use ratatui::{
    layout::{Alignment, Constraint, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame,
};

use crate::app::{App, AppState, CharResult};

pub fn draw(frame: &mut Frame, app: &App) {
    match app.state {
        AppState::Waiting | AppState::Running => draw_typing_screen(frame, app),
        AppState::Finished => draw_results_screen(frame, app),
    }
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
    let remaining = app.time_remaining();
    let secs = remaining.as_secs_f64();

    let timer_text = if app.state == AppState::Waiting {
        "Type to start...".to_string()
    } else {
        format!("{:.1}s", secs)
    };

    let color = if secs <= 5.0 && app.state == AppState::Running {
        Color::Red
    } else if secs <= 10.0 && app.state == AppState::Running {
        Color::Yellow
    } else {
        Color::Green
    };

    let timer = Paragraph::new(timer_text)
        .style(Style::default().fg(color).add_modifier(Modifier::BOLD))
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL).title("Timer"));

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
        Line::from(Span::styled(
            "Time's Up!",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
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
            "Press 'r' to restart | Press 'q' to quit",
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
