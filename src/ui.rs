//! User interface rendering for the typing test.
//!
//! This module handles all UI rendering using ratatui, including:
//! - Menu screen with test configuration options
//! - Typing screen with words, timer, and input display
//! - Results screen with statistics

use ratatui::{
    layout::{Alignment, Constraint, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use crate::app::{App, AppState, CharResult, MenuField, TestMode, TIME_OPTIONS, WORD_OPTIONS};

// =============================================================================
// Constants
// =============================================================================

/// ASCII art title displayed on the menu screen.
const ASCII_TITLE: &[&str] = &[
    "████████╗██╗   ██╗██████╗ ███████╗████████╗██╗   ██╗██╗",
    "╚══██╔══╝╚██╗ ██╔╝██╔══██╗██╔════╝╚══██╔══╝██║   ██║██║",
    "   ██║    ╚████╔╝ ██████╔╝█████╗     ██║   ██║   ██║██║",
    "   ██║     ╚██╔╝  ██╔═══╝ ██╔══╝     ██║   ██║   ██║██║",
    "   ██║      ██║   ██║     ███████╗   ██║   ╚██████╔╝██║",
    "   ╚═╝      ╚═╝   ╚═╝     ╚══════╝   ╚═╝    ╚═════╝ ╚═╝",
];

// =============================================================================
// Styles
// =============================================================================

/// Style for selected/highlighted menu items.
fn style_selected() -> Style {
    Style::default()
        .fg(Color::Yellow)
        .add_modifier(Modifier::BOLD)
}

/// Style for unselected menu items.
fn style_unselected() -> Style {
    Style::default().fg(Color::White)
}

/// Style for labels and help text.
fn style_label() -> Style {
    Style::default().fg(Color::DarkGray)
}

/// Style for correctly typed characters.
fn style_correct() -> Style {
    Style::default().fg(Color::Green)
}

/// Style for incorrectly typed characters.
fn style_incorrect() -> Style {
    Style::default().fg(Color::White).bg(Color::Red)
}

/// Style for characters not yet typed.
fn style_pending() -> Style {
    Style::default().fg(Color::DarkGray)
}

/// Style for the cursor position (current character to type).
fn style_cursor() -> Style {
    Style::default()
        .fg(Color::White)
        .add_modifier(Modifier::UNDERLINED)
}

// =============================================================================
// Main Draw Function
// =============================================================================

/// Main entry point for rendering the UI.
///
/// Dispatches to the appropriate screen based on app state.
pub fn draw(frame: &mut Frame, app: &App) {
    match app.state {
        AppState::Menu => draw_menu_screen(frame, app),
        AppState::Running => draw_typing_screen(frame, app),
        AppState::Finished => draw_results_screen(frame, app),
    }
}

// =============================================================================
// Menu Screen
// =============================================================================

/// Renders the menu screen with title and settings.
fn draw_menu_screen(frame: &mut Frame, app: &App) {
    let area = frame.area();

    let chunks = Layout::vertical([
        Constraint::Length(3),  // Top padding
        Constraint::Length(8),  // ASCII art title
        Constraint::Length(2),  // Spacing below title
        Constraint::Length(9),  // Settings box
        Constraint::Min(1),     // Spacer
        Constraint::Length(2),  // Help text
    ])
    .split(area);

    draw_title(frame, chunks[1]);
    draw_settings(frame, app, area, chunks[3]);
    draw_menu_help(frame, chunks[5]);
}

/// Renders the ASCII art title.
fn draw_title(frame: &mut Frame, area: Rect) {
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
    frame.render_widget(title, area);
}

/// Renders the settings box with mode, value, and start button.
fn draw_settings(frame: &mut Frame, app: &App, full_area: Rect, chunk: Rect) {
    let settings_width = 40u16;
    let settings_x = full_area.width.saturating_sub(settings_width) / 2;
    let settings_area = Rect::new(
        settings_x,
        chunk.y,
        settings_width.min(full_area.width),
        chunk.height,
    );

    let settings_block = Block::default()
        .borders(Borders::ALL)
        .border_style(style_label());
    frame.render_widget(settings_block, settings_area);

    let inner_area = Rect::new(
        settings_area.x + 2,
        settings_area.y + 1,
        settings_area.width.saturating_sub(4),
        settings_area.height.saturating_sub(2),
    );

    let rows = Layout::vertical([
        Constraint::Length(1),  // Spacer
        Constraint::Length(1),  // Mode
        Constraint::Length(1),  // Spacer
        Constraint::Length(1),  // Value
        Constraint::Length(1),  // Spacer
        Constraint::Length(1),  // Start
    ])
    .split(inner_area);

    draw_mode_row(frame, app, rows[1]);
    draw_value_row(frame, app, rows[3]);
    draw_start_row(frame, app, rows[5]);
}

/// Renders the mode selection row (Time/Words).
fn draw_mode_row(frame: &mut Frame, app: &App, area: Rect) {
    let is_selected = app.menu_field == MenuField::Mode;
    let style = if is_selected { style_selected() } else { style_unselected() };

    let mode_value = match app.test_mode {
        TestMode::Time => "Time",
        TestMode::Words => "Words",
    };

    let line = Line::from(vec![
        Span::styled("    Mode:  ", style_label()),
        Span::styled(format!("◄ {:^5} ►", mode_value), style),
    ]);

    frame.render_widget(Paragraph::new(line), area);
}

/// Renders the value selection row (duration or word count).
fn draw_value_row(frame: &mut Frame, app: &App, area: Rect) {
    let is_selected = app.menu_field == MenuField::Value;
    let style = if is_selected { style_selected() } else { style_unselected() };

    let (label, value_text) = match app.test_mode {
        TestMode::Time => {
            let secs = app.time_seconds();
            let display = if secs >= 60 {
                format!("{}m {}s", secs / 60, secs % 60)
            } else {
                format!("{}s", secs)
            };
            let left = if app.time_option_idx > 0 { "◄ " } else { "  " };
            let right = if app.time_option_idx < TIME_OPTIONS.len() - 1 { " ►" } else { "  " };
            ("    Time:", format!("{}{:^7}{}", left, display, right))
        }
        TestMode::Words => {
            let count = app.target_word_count();
            let left = if app.word_option_idx > 0 { "◄ " } else { "  " };
            let right = if app.word_option_idx < WORD_OPTIONS.len() - 1 { " ►" } else { "  " };
            ("   Words:", format!("{}{:^7}{}", left, count, right))
        }
    };

    let line = Line::from(vec![
        Span::styled(format!("{}  ", label), style_label()),
        Span::styled(value_text, style),
    ]);

    frame.render_widget(Paragraph::new(line), area);
}

/// Renders the start button row.
fn draw_start_row(frame: &mut Frame, app: &App, area: Rect) {
    let is_selected = app.menu_field == MenuField::Start;
    let style = if is_selected {
        Style::default()
            .fg(Color::Black)
            .bg(Color::Green)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::Green)
    };

    let line = Line::from(vec![
        Span::raw("         "),
        Span::styled("[ Start ]", style),
    ]);

    frame.render_widget(Paragraph::new(line), area);
}

/// Renders the help text at the bottom of the menu.
fn draw_menu_help(frame: &mut Frame, area: Rect) {
    let help = Paragraph::new(Line::from(Span::styled(
        "↑↓/jk: navigate  ←→/hl: change  Enter: start  q: quit",
        style_label(),
    )))
    .alignment(Alignment::Center);

    frame.render_widget(help, area);
}

// =============================================================================
// Typing Screen
// =============================================================================

/// Renders the main typing test screen.
fn draw_typing_screen(frame: &mut Frame, app: &App) {
    let area = frame.area();

    let horizontal_padding = 8u16;
    let padded_area = Rect::new(
        area.x + horizontal_padding,
        area.y,
        area.width.saturating_sub(horizontal_padding * 2),
        area.height,
    );

    let chunks = Layout::vertical([
        Constraint::Length(3),  // Timer
        Constraint::Length(3),  // Top padding
        Constraint::Length(5),  // Words (3 lines + border)
        Constraint::Length(3),  // Bottom padding
        Constraint::Length(3),  // Input
        Constraint::Min(0),     // Absorb remaining space
    ])
    .split(padded_area);

    draw_timer(frame, app, chunks[0]);
    draw_words(frame, app, chunks[2]);
    draw_input(frame, app, chunks[4]);
}

/// Renders the timer or progress indicator.
fn draw_timer(frame: &mut Frame, app: &App, area: Rect) {
    let (text, color, title) = if app.start_time.is_none() {
        ("Type to start...".to_string(), Color::Green, "Timer")
    } else {
        match app.test_mode {
            TestMode::Time => {
                let remaining = app.time_remaining().as_secs_f64();
                let color = if remaining <= 5.0 {
                    Color::Red
                } else if remaining <= 10.0 {
                    Color::Yellow
                } else {
                    Color::Green
                };
                (format!("{:.1}s", remaining), color, "Timer")
            }
            TestMode::Words => {
                let elapsed = app.time_elapsed().as_secs_f64();
                let text = format!(
                    "{}/{} words  |  {:.1}s",
                    app.current_word_idx,
                    app.words.len(),
                    elapsed
                );
                (text, Color::Cyan, "Progress")
            }
        }
    };

    let timer = Paragraph::new(text)
        .style(Style::default().fg(color).add_modifier(Modifier::BOLD))
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL).title(title));

    frame.render_widget(timer, area);
}

/// Renders the words to type with scrolling support.
///
/// Shows 3 lines at a time, scrolling when the user reaches line 2.
fn draw_words(frame: &mut Frame, app: &App, area: Rect) {
    let block = Block::default().borders(Borders::ALL).title("Words");
    let inner_area = block.inner(area);
    frame.render_widget(block, area);

    let side_padding = 4u16;
    let padded_inner = Rect::new(
        inner_area.x + side_padding,
        inner_area.y,
        inner_area.width.saturating_sub(side_padding * 2),
        inner_area.height,
    );

    let line_width = padded_inner.width as usize;
    if line_width == 0 {
        return;
    }

    let lines = build_word_lines(app, line_width);
    let current_line = find_current_line(&lines, app.current_word_idx);
    let first_visible = current_line.saturating_sub(1).max(if current_line >= 2 { current_line - 1 } else { 0 });

    let visible_lines: Vec<Line> = lines
        .iter()
        .skip(first_visible)
        .take(3)
        .map(|line_words| render_word_line(app, line_words))
        .collect();

    frame.render_widget(Paragraph::new(visible_lines), padded_inner);
}

/// Builds lines of words based on available width.
///
/// Returns a vector of lines, where each line is a vector of (word_index, word) tuples.
fn build_word_lines(app: &App, line_width: usize) -> Vec<Vec<(usize, String)>> {
    let mut lines = Vec::new();
    let mut current_line = Vec::new();
    let mut current_width = 0;

    for (idx, word) in app.words.iter().enumerate() {
        let word_len = word.len() + 1; // +1 for space

        if current_width + word_len > line_width && !current_line.is_empty() {
            lines.push(current_line);
            current_line = Vec::new();
            current_width = 0;
        }

        current_line.push((idx, word.clone()));
        current_width += word_len;
    }

    if !current_line.is_empty() {
        lines.push(current_line);
    }

    lines
}

/// Finds which line contains the current word.
fn find_current_line(lines: &[Vec<(usize, String)>], current_word_idx: usize) -> usize {
    lines
        .iter()
        .position(|line| line.iter().any(|(idx, _)| *idx == current_word_idx))
        .unwrap_or(0)
}

/// Renders a single line of words with appropriate styling.
fn render_word_line(app: &App, line_words: &[(usize, String)]) -> Line<'static> {
    let mut spans = Vec::new();

    for (idx, word) in line_words {
        if *idx < app.current_word_idx {
            // Completed word
            let style = if app.word_completed_correctly(*idx) {
                style_correct()
            } else {
                Style::default().fg(Color::Red)
            };
            spans.push(Span::styled(word.clone(), style));
            spans.push(Span::raw(" "));
        } else if *idx == app.current_word_idx {
            // Current word - render character by character
            spans.extend(render_current_word(app, word));
            spans.push(Span::raw(" "));
        } else {
            // Upcoming word
            spans.push(Span::styled(word.clone(), style_pending()));
            spans.push(Span::raw(" "));
        }
    }

    Line::from(spans)
}

/// Renders the current word being typed with per-character styling.
fn render_current_word(app: &App, word: &str) -> Vec<Span<'static>> {
    let mut spans = Vec::new();
    let typed_len = app.typed_input.len();
    let word_idx = app.current_word_idx;

    for (char_idx, c) in word.chars().enumerate() {
        let style = if char_idx < typed_len {
            match app.word_char_results[word_idx].get(char_idx) {
                Some(CharResult::Correct) => style_correct(),
                Some(CharResult::Incorrect) => style_incorrect(),
                None => style_pending(),
            }
        } else if char_idx == typed_len {
            style_cursor()
        } else {
            style_pending()
        };
        spans.push(Span::styled(c.to_string(), style));
    }

    // Extra characters typed beyond word length
    if typed_len > word.len() {
        let extra: String = app.typed_input.chars().skip(word.len()).collect();
        spans.push(Span::styled(extra, style_incorrect()));
    }

    spans
}

/// Renders the input field showing what the user is typing.
fn draw_input(frame: &mut Frame, app: &App, area: Rect) {
    let input = Paragraph::new(format!("> {}_", app.typed_input))
        .style(Style::default().fg(Color::White))
        .block(Block::default().borders(Borders::ALL).title("Input"));

    frame.render_widget(input, area);
}

// =============================================================================
// Results Screen
// =============================================================================

/// Renders the results screen with test statistics.
fn draw_results_screen(frame: &mut Frame, app: &App) {
    let area = frame.area();
    let stats = app.calculate_stats();

    let text = vec![
        Line::from(""),
        stat_line("WPM: ", format!("{:.0}", stats.wpm), Color::Cyan),
        Line::from(""),
        stat_line("Accuracy: ", format!("{:.1}%", stats.accuracy), Color::Green),
        Line::from(""),
        stat_line("Total Errors: ", format!("{}", stats.total_errors), Color::Red),
        Line::from(""),
        stat_line("Words with mistakes: ", format!("{}", stats.words_with_errors), Color::Red),
        Line::from(""),
        Line::from(vec![
            Span::raw("Characters: "),
            Span::styled(
                format!("{}", stats.correct_chars),
                Style::default().fg(Color::Green).add_modifier(Modifier::BOLD),
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
            style_label(),
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

/// Helper to create a statistics line with colored value.
fn stat_line(label: &str, value: String, color: Color) -> Line<'static> {
    Line::from(vec![
        Span::raw(label.to_string()),
        Span::styled(
            value,
            Style::default().fg(color).add_modifier(Modifier::BOLD),
        ),
    ])
}
