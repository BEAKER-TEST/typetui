use std::time::{Duration, Instant};

use crate::words::generate_words;

pub const TIME_OPTIONS: &[u32] = &[30, 60, 90, 120];
pub const WORD_OPTIONS: &[u32] = &[10, 25, 50, 100, 150, 200];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AppState {
    Menu,
    Running,
    Finished,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TestMode {
    Time,
    Words,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MenuField {
    Mode,
    Value,
    Start,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CharResult {
    Correct,
    Incorrect,
}

pub struct Stats {
    pub wpm: f64,
    pub accuracy: f64,
    pub total_errors: usize,
    pub words_with_errors: usize,
    pub total_chars_typed: usize,
    pub correct_chars: usize,
}

pub struct App {
    // Test state
    pub words: Vec<String>,
    pub current_word_idx: usize,
    pub typed_input: String,
    pub word_char_results: Vec<Vec<CharResult>>,
    pub current_word_has_error: bool,
    pub words_with_errors: Vec<usize>,
    pub total_chars_typed: usize,
    pub correct_chars: usize,
    pub start_time: Option<Instant>,
    pub duration: Duration,
    pub state: AppState,

    // Menu state
    pub menu_field: MenuField,
    pub test_mode: TestMode,
    pub time_option_idx: usize,
    pub word_option_idx: usize,
}

impl App {
    pub fn new() -> Self {
        Self {
            words: Vec::new(),
            current_word_idx: 0,
            typed_input: String::new(),
            word_char_results: Vec::new(),
            current_word_has_error: false,
            words_with_errors: Vec::new(),
            total_chars_typed: 0,
            correct_chars: 0,
            start_time: None,
            duration: Duration::from_secs(30),
            state: AppState::Menu,

            menu_field: MenuField::Mode,
            test_mode: TestMode::Time,
            time_option_idx: 0,  // 30s
            word_option_idx: 1,  // 25 words
        }
    }

    pub fn time_seconds(&self) -> u32 {
        TIME_OPTIONS[self.time_option_idx]
    }

    pub fn target_word_count(&self) -> u32 {
        WORD_OPTIONS[self.word_option_idx]
    }

    #[cfg(test)]
    pub fn with_words(words: Vec<String>) -> Self {
        let word_char_results = vec![Vec::new(); words.len()];

        Self {
            words,
            current_word_idx: 0,
            typed_input: String::new(),
            word_char_results,
            current_word_has_error: false,
            words_with_errors: Vec::new(),
            total_chars_typed: 0,
            correct_chars: 0,
            start_time: None,
            duration: Duration::from_secs(30),
            state: AppState::Running,

            menu_field: MenuField::Mode,
            test_mode: TestMode::Time,
            time_option_idx: 0,
            word_option_idx: 1,
        }
    }

    pub fn reset(&mut self) {
        self.words.clear();
        self.current_word_idx = 0;
        self.typed_input.clear();
        self.word_char_results.clear();
        self.current_word_has_error = false;
        self.words_with_errors.clear();
        self.total_chars_typed = 0;
        self.correct_chars = 0;
        self.start_time = None;
        self.state = AppState::Menu;
        self.menu_field = MenuField::Mode;
    }

    // Menu navigation
    pub fn menu_up(&mut self) {
        if self.state != AppState::Menu {
            return;
        }
        self.menu_field = match self.menu_field {
            MenuField::Mode => MenuField::Mode,
            MenuField::Value => MenuField::Mode,
            MenuField::Start => MenuField::Value,
        };
    }

    pub fn menu_down(&mut self) {
        if self.state != AppState::Menu {
            return;
        }
        self.menu_field = match self.menu_field {
            MenuField::Mode => MenuField::Value,
            MenuField::Value => MenuField::Start,
            MenuField::Start => MenuField::Start,
        };
    }

    pub fn menu_left(&mut self) {
        if self.state != AppState::Menu {
            return;
        }
        match self.menu_field {
            MenuField::Mode => {
                self.test_mode = match self.test_mode {
                    TestMode::Time => TestMode::Words,
                    TestMode::Words => TestMode::Time,
                };
            }
            MenuField::Value => match self.test_mode {
                TestMode::Time => {
                    if self.time_option_idx > 0 {
                        self.time_option_idx -= 1;
                    }
                }
                TestMode::Words => {
                    if self.word_option_idx > 0 {
                        self.word_option_idx -= 1;
                    }
                }
            },
            MenuField::Start => {}
        }
    }

    pub fn menu_right(&mut self) {
        if self.state != AppState::Menu {
            return;
        }
        match self.menu_field {
            MenuField::Mode => {
                self.test_mode = match self.test_mode {
                    TestMode::Time => TestMode::Words,
                    TestMode::Words => TestMode::Time,
                };
            }
            MenuField::Value => match self.test_mode {
                TestMode::Time => {
                    if self.time_option_idx < TIME_OPTIONS.len() - 1 {
                        self.time_option_idx += 1;
                    }
                }
                TestMode::Words => {
                    if self.word_option_idx < WORD_OPTIONS.len() - 1 {
                        self.word_option_idx += 1;
                    }
                }
            },
            MenuField::Start => {}
        }
    }

    pub fn menu_select(&mut self) {
        if self.state != AppState::Menu {
            return;
        }

        match self.menu_field {
            MenuField::Mode | MenuField::Value => {
                // Left/right already handles these, Enter just moves down
                self.menu_down();
            }
            MenuField::Start => {
                self.start_test();
            }
        }
    }

    fn start_test(&mut self) {
        let word_count = match self.test_mode {
            TestMode::Time => 200, // Plenty of words for timed mode
            TestMode::Words => self.target_word_count() as usize,
        };

        self.words = generate_words(word_count);
        self.word_char_results = vec![Vec::new(); self.words.len()];
        self.current_word_idx = 0;
        self.typed_input.clear();
        self.current_word_has_error = false;
        self.words_with_errors.clear();
        self.total_chars_typed = 0;
        self.correct_chars = 0;
        self.start_time = None;
        self.duration = Duration::from_secs(self.time_seconds() as u64);
        self.state = AppState::Running;
    }

    pub fn time_elapsed(&self) -> Duration {
        match self.start_time {
            Some(start) => start.elapsed(),
            None => Duration::ZERO,
        }
    }

    pub fn time_remaining(&self) -> Duration {
        match self.start_time {
            Some(start) => {
                let elapsed = start.elapsed();
                if elapsed >= self.duration {
                    Duration::ZERO
                } else {
                    self.duration - elapsed
                }
            }
            None => self.duration,
        }
    }

    pub fn tick(&mut self) {
        if self.state != AppState::Running {
            return;
        }

        // Only Time mode ends on timer
        if self.test_mode == TestMode::Time {
            if let Some(start) = self.start_time {
                if start.elapsed() >= self.duration {
                    self.state = AppState::Finished;
                }
            }
        }
    }

    pub fn on_char(&mut self, c: char) {
        if self.state != AppState::Running {
            return;
        }

        // Start timer on first character
        if self.start_time.is_none() {
            self.start_time = Some(Instant::now());
        }

        if self.current_word_idx >= self.words.len() {
            return;
        }

        let current_word = &self.words[self.current_word_idx];
        let char_pos = self.typed_input.len();

        self.total_chars_typed += 1;

        let expected_char = current_word.chars().nth(char_pos);
        let is_correct = expected_char == Some(c);

        if is_correct {
            self.correct_chars += 1;
            self.word_char_results[self.current_word_idx].push(CharResult::Correct);
        } else {
            self.current_word_has_error = true;
            self.word_char_results[self.current_word_idx].push(CharResult::Incorrect);
        }

        self.typed_input.push(c);
    }

    pub fn on_backspace(&mut self) {
        if self.state != AppState::Running || self.typed_input.is_empty() {
            return;
        }

        self.typed_input.pop();
        if !self.word_char_results[self.current_word_idx].is_empty() {
            self.word_char_results[self.current_word_idx].pop();
        }
    }

    pub fn on_space(&mut self) {
        if self.state != AppState::Running {
            return;
        }

        // Don't advance if timer hasn't started
        if self.start_time.is_none() {
            return;
        }

        if self.current_word_idx >= self.words.len() {
            return;
        }

        if self.current_word_has_error {
            self.words_with_errors.push(self.current_word_idx);
        }

        self.current_word_idx += 1;
        self.typed_input.clear();
        self.current_word_has_error = false;

        // In Words mode, check if all words completed
        if self.test_mode == TestMode::Words && self.current_word_idx >= self.words.len() {
            self.state = AppState::Finished;
        }
    }

    pub fn calculate_stats(&self) -> Stats {
        let elapsed_secs = match self.start_time {
            Some(start) => {
                let elapsed = start.elapsed();
                match self.test_mode {
                    TestMode::Time => {
                        if elapsed > self.duration {
                            self.duration.as_secs_f64()
                        } else {
                            elapsed.as_secs_f64()
                        }
                    }
                    TestMode::Words => elapsed.as_secs_f64(),
                }
            }
            None => 0.0,
        };

        let elapsed_mins = elapsed_secs / 60.0;

        let wpm = if elapsed_mins > 0.0 {
            (self.correct_chars as f64 / 5.0) / elapsed_mins
        } else {
            0.0
        };

        let accuracy = if self.total_chars_typed > 0 {
            (self.correct_chars as f64 / self.total_chars_typed as f64) * 100.0
        } else {
            100.0
        };

        let total_errors = self.total_chars_typed - self.correct_chars;

        Stats {
            wpm,
            accuracy,
            total_errors,
            words_with_errors: self.words_with_errors.len(),
            total_chars_typed: self.total_chars_typed,
            correct_chars: self.correct_chars,
        }
    }

    pub fn word_completed_correctly(&self, word_idx: usize) -> bool {
        if word_idx >= self.current_word_idx {
            return false;
        }
        !self.words_with_errors.contains(&word_idx)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_words() -> Vec<String> {
        vec!["the".to_string(), "quick".to_string(), "brown".to_string()]
    }

    fn running_app() -> App {
        App::with_words(test_words())
    }

    #[test]
    fn test_initial_state_is_menu() {
        let app = App::new();
        assert_eq!(app.state, AppState::Menu);
        assert_eq!(app.menu_field, MenuField::Mode);
        assert_eq!(app.test_mode, TestMode::Time);
        assert_eq!(app.time_option_idx, 0); // 30s default
        assert_eq!(app.word_option_idx, 1); // 25 words default
    }

    #[test]
    fn test_menu_navigation() {
        let mut app = App::new();

        assert_eq!(app.menu_field, MenuField::Mode);

        app.menu_down();
        assert_eq!(app.menu_field, MenuField::Value);

        app.menu_down();
        assert_eq!(app.menu_field, MenuField::Start);

        app.menu_down(); // Should stay at Start
        assert_eq!(app.menu_field, MenuField::Start);

        app.menu_up();
        assert_eq!(app.menu_field, MenuField::Value);

        app.menu_up();
        assert_eq!(app.menu_field, MenuField::Mode);

        app.menu_up(); // Should stay at Mode
        assert_eq!(app.menu_field, MenuField::Mode);
    }

    #[test]
    fn test_menu_toggle_mode() {
        let mut app = App::new();
        assert_eq!(app.menu_field, MenuField::Mode);
        assert_eq!(app.test_mode, TestMode::Time);

        app.menu_right();
        assert_eq!(app.test_mode, TestMode::Words);

        app.menu_left();
        assert_eq!(app.test_mode, TestMode::Time);
    }

    #[test]
    fn test_menu_value_selection_time() {
        let mut app = App::new();
        app.menu_field = MenuField::Value;

        assert_eq!(app.time_option_idx, 0);
        assert_eq!(app.time_seconds(), 30);

        app.menu_right();
        assert_eq!(app.time_option_idx, 1);
        assert_eq!(app.time_seconds(), 60);

        app.menu_right();
        assert_eq!(app.time_option_idx, 2);
        assert_eq!(app.time_seconds(), 90);

        app.menu_right();
        assert_eq!(app.time_option_idx, 3);
        assert_eq!(app.time_seconds(), 120);

        // Should stay at max
        app.menu_right();
        assert_eq!(app.time_option_idx, 3);

        app.menu_left();
        assert_eq!(app.time_option_idx, 2);
        assert_eq!(app.time_seconds(), 90);
    }

    #[test]
    fn test_menu_value_selection_words() {
        let mut app = App::new();
        app.test_mode = TestMode::Words;
        app.menu_field = MenuField::Value;

        assert_eq!(app.word_option_idx, 1); // Default 25
        assert_eq!(app.target_word_count(), 25);

        app.menu_left();
        assert_eq!(app.word_option_idx, 0);
        assert_eq!(app.target_word_count(), 10);

        // Should stay at min
        app.menu_left();
        assert_eq!(app.word_option_idx, 0);

        app.menu_right();
        assert_eq!(app.word_option_idx, 1);
        assert_eq!(app.target_word_count(), 25);

        app.menu_right();
        assert_eq!(app.target_word_count(), 50);

        app.menu_right();
        assert_eq!(app.target_word_count(), 100);

        app.menu_right();
        assert_eq!(app.target_word_count(), 150);

        app.menu_right();
        assert_eq!(app.target_word_count(), 200);

        // Should stay at max
        app.menu_right();
        assert_eq!(app.target_word_count(), 200);
    }

    #[test]
    fn test_menu_start_begins_test() {
        let mut app = App::new();
        app.menu_field = MenuField::Start;

        app.menu_select();

        assert_eq!(app.state, AppState::Running);
        assert!(!app.words.is_empty());
    }

    #[test]
    fn test_first_char_starts_timer() {
        let mut app = running_app();

        assert!(app.start_time.is_none());

        app.on_char('t');

        assert!(app.start_time.is_some());
    }

    #[test]
    fn test_correct_typing() {
        let mut app = running_app();

        app.on_char('t');
        app.on_char('h');
        app.on_char('e');

        assert_eq!(app.typed_input, "the");
        assert_eq!(app.total_chars_typed, 3);
        assert_eq!(app.correct_chars, 3);
        assert!(!app.current_word_has_error);

        assert_eq!(app.word_char_results[0].len(), 3);
        assert!(app.word_char_results[0]
            .iter()
            .all(|r| *r == CharResult::Correct));
    }

    #[test]
    fn test_incorrect_typing() {
        let mut app = running_app();

        app.on_char('t');
        app.on_char('h');
        app.on_char('x');

        assert_eq!(app.typed_input, "thx");
        assert_eq!(app.total_chars_typed, 3);
        assert_eq!(app.correct_chars, 2);
        assert!(app.current_word_has_error);

        assert_eq!(app.word_char_results[0][2], CharResult::Incorrect);
    }

    #[test]
    fn test_space_advances_word() {
        let mut app = running_app();

        app.on_char('t');
        app.on_char('h');
        app.on_char('e');
        app.on_space();

        assert_eq!(app.current_word_idx, 1);
        assert!(app.typed_input.is_empty());
        assert!(!app.current_word_has_error);
    }

    #[test]
    fn test_space_does_not_advance_before_typing() {
        let mut app = running_app();

        app.on_space();

        assert_eq!(app.current_word_idx, 0);
        assert!(app.start_time.is_none());
    }

    #[test]
    fn test_word_with_error_tracked() {
        let mut app = running_app();

        app.on_char('t');
        app.on_char('h');
        app.on_char('x');
        app.on_space();

        assert_eq!(app.words_with_errors.len(), 1);
        assert_eq!(app.words_with_errors[0], 0);
        assert!(!app.word_completed_correctly(0));
    }

    #[test]
    fn test_word_completed_correctly() {
        let mut app = running_app();

        app.on_char('t');
        app.on_char('h');
        app.on_char('e');
        app.on_space();

        assert!(app.word_completed_correctly(0));
        assert!(!app.word_completed_correctly(1));
    }

    #[test]
    fn test_backspace() {
        let mut app = running_app();

        app.on_char('t');
        app.on_char('h');
        app.on_char('x');

        assert_eq!(app.typed_input, "thx");
        assert_eq!(app.word_char_results[0].len(), 3);

        app.on_backspace();

        assert_eq!(app.typed_input, "th");
        assert_eq!(app.word_char_results[0].len(), 2);
    }

    #[test]
    fn test_backspace_on_empty_does_nothing() {
        let mut app = running_app();
        app.on_char('t');
        app.on_backspace();
        app.on_backspace();

        assert!(app.typed_input.is_empty());
    }

    #[test]
    fn test_time_remaining_before_start() {
        let app = running_app();

        assert_eq!(app.time_remaining(), Duration::from_secs(30));
    }

    #[test]
    fn test_time_remaining_after_start() {
        let mut app = running_app();
        app.on_char('t');

        assert!(app.time_remaining() <= Duration::from_secs(30));
        assert!(app.time_remaining() > Duration::from_secs(29));
    }

    #[test]
    fn test_stats_accuracy_all_correct() {
        let mut app = running_app();

        app.on_char('t');
        app.on_char('h');
        app.on_char('e');

        let stats = app.calculate_stats();
        assert_eq!(stats.accuracy, 100.0);
        assert_eq!(stats.total_errors, 0);
    }

    #[test]
    fn test_stats_accuracy_with_errors() {
        let mut app = running_app();

        app.on_char('t');
        app.on_char('h');
        app.on_char('x');
        app.on_char('e');

        let stats = app.calculate_stats();
        assert_eq!(stats.correct_chars, 2);
        assert_eq!(stats.total_chars_typed, 4);
        assert_eq!(stats.accuracy, 50.0);
        assert_eq!(stats.total_errors, 2);
    }

    #[test]
    fn test_stats_words_with_errors() {
        let mut app = running_app();

        app.on_char('t');
        app.on_char('h');
        app.on_char('e');
        app.on_space();

        app.on_char('q');
        app.on_char('x');
        app.on_space();

        let stats = app.calculate_stats();
        assert_eq!(stats.words_with_errors, 1);
    }

    #[test]
    fn test_reset_returns_to_menu() {
        let mut app = running_app();

        app.on_char('t');
        app.on_char('h');
        app.on_char('e');
        app.on_space();

        app.reset();

        assert_eq!(app.state, AppState::Menu);
        assert_eq!(app.current_word_idx, 0);
        assert!(app.typed_input.is_empty());
        assert_eq!(app.total_chars_typed, 0);
        assert_eq!(app.correct_chars, 0);
        assert!(app.start_time.is_none());
        assert!(app.words_with_errors.is_empty());
    }

    #[test]
    fn test_typing_after_finished_does_nothing() {
        let mut app = running_app();
        app.state = AppState::Finished;

        app.on_char('t');

        assert!(app.typed_input.is_empty());
        assert_eq!(app.total_chars_typed, 0);
    }

    #[test]
    fn test_space_after_finished_does_nothing() {
        let mut app = running_app();
        app.on_char('t');
        app.state = AppState::Finished;

        let idx_before = app.current_word_idx;
        app.on_space();

        assert_eq!(app.current_word_idx, idx_before);
    }

    #[test]
    fn test_multiple_words_typed() {
        let mut app = running_app();

        for c in "the".chars() {
            app.on_char(c);
        }
        app.on_space();

        for c in "quick".chars() {
            app.on_char(c);
        }
        app.on_space();

        assert_eq!(app.current_word_idx, 2);
        assert_eq!(app.correct_chars, 8);
        assert!(app.word_completed_correctly(0));
        assert!(app.word_completed_correctly(1));
    }

    #[test]
    fn test_words_mode_finishes_on_last_word() {
        let mut app = App::with_words(test_words());
        app.test_mode = TestMode::Words;

        // Type all three words
        for c in "the".chars() {
            app.on_char(c);
        }
        app.on_space();

        for c in "quick".chars() {
            app.on_char(c);
        }
        app.on_space();

        for c in "brown".chars() {
            app.on_char(c);
        }
        app.on_space();

        assert_eq!(app.state, AppState::Finished);
    }

    #[test]
    fn test_time_mode_does_not_finish_on_last_word() {
        let mut app = App::with_words(test_words());
        app.test_mode = TestMode::Time;

        for c in "the".chars() {
            app.on_char(c);
        }
        app.on_space();

        for c in "quick".chars() {
            app.on_char(c);
        }
        app.on_space();

        for c in "brown".chars() {
            app.on_char(c);
        }
        app.on_space();

        assert_eq!(app.state, AppState::Running);
    }
}
