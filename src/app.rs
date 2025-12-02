//! Application state and logic for the typing test.
//!
//! This module contains the core business logic including:
//! - Test state management (words, progress, timing)
//! - Menu navigation
//! - Input handling (typing, backspace, word advancement)
//! - Statistics calculation

use std::time::{Duration, Instant};

use crate::words::generate_words;

// =============================================================================
// Constants
// =============================================================================

/// Available time options for timed mode (in seconds).
pub const TIME_OPTIONS: &[u32] = &[30, 60, 90, 120];

/// Available word count options for words mode.
pub const WORD_OPTIONS: &[u32] = &[10, 25, 50, 100, 150, 200];

// =============================================================================
// Types
// =============================================================================

/// The current state of the application.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AppState {
    /// User is on the menu screen selecting test options.
    Menu,
    /// User is actively taking the typing test.
    Running,
    /// Test is complete and results are being displayed.
    Finished,
}

/// The type of typing test.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TestMode {
    /// Test runs for a fixed duration.
    Time,
    /// Test runs until a fixed number of words are typed.
    Words,
}

/// The currently selected field in the menu.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MenuField {
    /// Mode selection (Time/Words).
    Mode,
    /// Value selection (duration or word count).
    Value,
    /// Start button.
    Start,
}

/// Result of typing a single character.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CharResult {
    /// Character matched the expected character.
    Correct,
    /// Character did not match the expected character.
    Incorrect,
}

/// Statistics calculated at the end of a typing test.
#[derive(Debug)]
pub struct Stats {
    /// Words per minute (based on 5 characters = 1 word).
    pub wpm: f64,
    /// Percentage of correctly typed characters.
    pub accuracy: f64,
    /// Total number of incorrectly typed characters.
    pub total_errors: usize,
    /// Number of words that contained at least one error.
    pub words_with_errors: usize,
    /// Total characters typed (correct + incorrect).
    pub total_chars_typed: usize,
    /// Number of correctly typed characters.
    pub correct_chars: usize,
}

// =============================================================================
// App State
// =============================================================================

/// Main application state container.
///
/// Holds all state for the typing test including:
/// - The list of words to type
/// - Current progress through the test
/// - Timing information
/// - Menu selections
pub struct App {
    /// List of words to type in the current test.
    pub words: Vec<String>,
    /// Index of the word currently being typed.
    pub current_word_idx: usize,
    /// Characters typed for the current word.
    pub typed_input: String,
    /// Per-character results for each word (correct/incorrect).
    pub word_char_results: Vec<Vec<CharResult>>,
    /// Whether the current word has any errors.
    pub current_word_has_error: bool,
    /// Indices of words that were completed with errors.
    pub words_with_errors: Vec<usize>,
    /// Total characters typed across all words.
    pub total_chars_typed: usize,
    /// Total correctly typed characters.
    pub correct_chars: usize,
    /// When the test started (first character typed).
    pub start_time: Option<Instant>,
    /// Duration limit for timed mode.
    pub duration: Duration,
    /// Current application state.
    pub state: AppState,

    /// Currently selected menu field.
    pub menu_field: MenuField,
    /// Selected test mode.
    pub test_mode: TestMode,
    /// Index into `TIME_OPTIONS` for selected duration.
    pub time_option_idx: usize,
    /// Index into `WORD_OPTIONS` for selected word count.
    pub word_option_idx: usize,
}

impl App {
    /// Creates a new App instance with default settings.
    ///
    /// Defaults to:
    /// - Time mode with 30 seconds
    /// - 25 words for words mode
    /// - Menu state
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
            time_option_idx: 0,
            word_option_idx: 1,
        }
    }

    /// Creates an App with predefined words for testing purposes.
    #[cfg(test)]
    pub fn with_words(words: Vec<String>) -> Self {
        let word_char_results = vec![Vec::new(); words.len()];
        Self {
            words,
            word_char_results,
            state: AppState::Running,
            ..Self::new()
        }
    }

    // =========================================================================
    // Menu Navigation
    // =========================================================================

    /// Moves menu selection up.
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

    /// Moves menu selection down.
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

    /// Handles left arrow in menu (toggle mode or decrement value).
    pub fn menu_left(&mut self) {
        if self.state != AppState::Menu {
            return;
        }
        match self.menu_field {
            MenuField::Mode => self.toggle_mode(),
            MenuField::Value => self.decrement_value(),
            MenuField::Start => {}
        }
    }

    /// Handles right arrow in menu (toggle mode or increment value).
    pub fn menu_right(&mut self) {
        if self.state != AppState::Menu {
            return;
        }
        match self.menu_field {
            MenuField::Mode => self.toggle_mode(),
            MenuField::Value => self.increment_value(),
            MenuField::Start => {}
        }
    }

    /// Toggles between Time and Words mode.
    fn toggle_mode(&mut self) {
        self.test_mode = match self.test_mode {
            TestMode::Time => TestMode::Words,
            TestMode::Words => TestMode::Time,
        };
    }

    /// Decrements the current value option (time or word count).
    fn decrement_value(&mut self) {
        match self.test_mode {
            TestMode::Time => {
                self.time_option_idx = self.time_option_idx.saturating_sub(1);
            }
            TestMode::Words => {
                self.word_option_idx = self.word_option_idx.saturating_sub(1);
            }
        }
    }

    /// Increments the current value option (time or word count).
    fn increment_value(&mut self) {
        match self.test_mode {
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
        }
    }

    // =========================================================================
    // Test Control
    // =========================================================================

    /// Starts a new typing test with current settings.
    ///
    /// Generates random words and resets all test state.
    pub fn start_test(&mut self) {
        let word_count = match self.test_mode {
            TestMode::Time => 200,
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

    /// Resets the app back to the menu state.
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

    /// Called each frame to check for time-based test completion.
    pub fn tick(&mut self) {
        if self.state != AppState::Running {
            return;
        }

        if self.test_mode == TestMode::Time {
            if let Some(start) = self.start_time {
                if start.elapsed() >= self.duration {
                    self.state = AppState::Finished;
                }
            }
        }
    }

    // =========================================================================
    // Typing Input
    // =========================================================================

    /// Handles a character being typed.
    ///
    /// - Starts the timer on first character
    /// - Records whether the character was correct or incorrect
    /// - Updates statistics
    pub fn on_char(&mut self, c: char) {
        if self.state != AppState::Running {
            return;
        }

        if self.start_time.is_none() {
            self.start_time = Some(Instant::now());
        }

        if self.current_word_idx >= self.words.len() {
            return;
        }

        let current_word = &self.words[self.current_word_idx];
        let char_pos = self.typed_input.len();
        let expected_char = current_word.chars().nth(char_pos);
        let is_correct = expected_char == Some(c);

        self.total_chars_typed += 1;

        if is_correct {
            self.correct_chars += 1;
            self.word_char_results[self.current_word_idx].push(CharResult::Correct);
        } else {
            self.current_word_has_error = true;
            self.word_char_results[self.current_word_idx].push(CharResult::Incorrect);
        }

        self.typed_input.push(c);
    }

    /// Handles backspace being pressed.
    ///
    /// Removes the last typed character and its result.
    pub fn on_backspace(&mut self) {
        if self.state != AppState::Running || self.typed_input.is_empty() {
            return;
        }

        self.typed_input.pop();
        if !self.word_char_results[self.current_word_idx].is_empty() {
            self.word_char_results[self.current_word_idx].pop();
        }
    }

    /// Handles space being pressed.
    ///
    /// If the word is complete, advances to the next word.
    /// If the word is incomplete, treats space as an incorrect character.
    pub fn on_space(&mut self) {
        if self.state != AppState::Running || self.start_time.is_none() {
            return;
        }

        if self.current_word_idx >= self.words.len() {
            return;
        }

        let current_word = &self.words[self.current_word_idx];

        // If word is not fully typed, treat space as an incorrect character
        if self.typed_input.len() < current_word.len() {
            self.on_char(' ');
            return;
        }

        // Word is complete (or over-typed), advance to next word
        let has_error = self.current_word_has_error || self.typed_input.len() > current_word.len();
        if has_error {
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

    // =========================================================================
    // Accessors
    // =========================================================================

    /// Returns the selected time duration in seconds.
    pub fn time_seconds(&self) -> u32 {
        TIME_OPTIONS[self.time_option_idx]
    }

    /// Returns the selected target word count.
    pub fn target_word_count(&self) -> u32 {
        WORD_OPTIONS[self.word_option_idx]
    }

    /// Returns the elapsed time since the test started.
    pub fn time_elapsed(&self) -> Duration {
        self.start_time.map_or(Duration::ZERO, |s| s.elapsed())
    }

    /// Returns the remaining time in timed mode.
    pub fn time_remaining(&self) -> Duration {
        match self.start_time {
            Some(start) => self.duration.saturating_sub(start.elapsed()),
            None => self.duration,
        }
    }

    /// Returns whether a completed word was typed correctly.
    pub fn word_completed_correctly(&self, word_idx: usize) -> bool {
        word_idx < self.current_word_idx && !self.words_with_errors.contains(&word_idx)
    }

    // =========================================================================
    // Statistics
    // =========================================================================

    /// Calculates and returns test statistics.
    ///
    /// WPM is calculated as (correct_chars / 5) / minutes.
    /// Accuracy is (correct_chars / total_chars) * 100.
    pub fn calculate_stats(&self) -> Stats {
        let elapsed_secs = self.calculate_elapsed_secs();
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

        Stats {
            wpm,
            accuracy,
            total_errors: self.total_chars_typed - self.correct_chars,
            words_with_errors: self.words_with_errors.len(),
            total_chars_typed: self.total_chars_typed,
            correct_chars: self.correct_chars,
        }
    }

    /// Calculates elapsed seconds, capped at duration for timed mode.
    fn calculate_elapsed_secs(&self) -> f64 {
        match self.start_time {
            Some(start) => {
                let elapsed = start.elapsed();
                match self.test_mode {
                    TestMode::Time => elapsed.min(self.duration).as_secs_f64(),
                    TestMode::Words => elapsed.as_secs_f64(),
                }
            }
            None => 0.0,
        }
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    fn test_words() -> Vec<String> {
        vec!["the".into(), "quick".into(), "brown".into()]
    }

    fn running_app() -> App {
        App::with_words(test_words())
    }

    // -------------------------------------------------------------------------
    // Menu Tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_initial_state_is_menu() {
        let app = App::new();
        assert_eq!(app.state, AppState::Menu);
        assert_eq!(app.menu_field, MenuField::Mode);
        assert_eq!(app.test_mode, TestMode::Time);
        assert_eq!(app.time_option_idx, 0);
        assert_eq!(app.word_option_idx, 1);
    }

    #[test]
    fn test_menu_navigation() {
        let mut app = App::new();
        assert_eq!(app.menu_field, MenuField::Mode);

        app.menu_down();
        assert_eq!(app.menu_field, MenuField::Value);

        app.menu_down();
        assert_eq!(app.menu_field, MenuField::Start);

        app.menu_down();
        assert_eq!(app.menu_field, MenuField::Start);

        app.menu_up();
        assert_eq!(app.menu_field, MenuField::Value);

        app.menu_up();
        assert_eq!(app.menu_field, MenuField::Mode);

        app.menu_up();
        assert_eq!(app.menu_field, MenuField::Mode);
    }

    #[test]
    fn test_menu_toggle_mode() {
        let mut app = App::new();
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

        assert_eq!(app.time_seconds(), 30);

        app.menu_right();
        assert_eq!(app.time_seconds(), 60);

        app.menu_right();
        assert_eq!(app.time_seconds(), 90);

        app.menu_right();
        assert_eq!(app.time_seconds(), 120);

        app.menu_right();
        assert_eq!(app.time_seconds(), 120); // stays at max

        app.menu_left();
        assert_eq!(app.time_seconds(), 90);
    }

    #[test]
    fn test_menu_value_selection_words() {
        let mut app = App::new();
        app.test_mode = TestMode::Words;
        app.menu_field = MenuField::Value;

        assert_eq!(app.target_word_count(), 25);

        app.menu_left();
        assert_eq!(app.target_word_count(), 10);

        app.menu_left();
        assert_eq!(app.target_word_count(), 10); // stays at min

        app.menu_right();
        assert_eq!(app.target_word_count(), 25);

        // Navigate to max
        for _ in 0..10 {
            app.menu_right();
        }
        assert_eq!(app.target_word_count(), 200);
    }

    #[test]
    fn test_menu_start_begins_test() {
        let mut app = App::new();
        app.start_test();

        assert_eq!(app.state, AppState::Running);
        assert!(!app.words.is_empty());
    }

    // -------------------------------------------------------------------------
    // Typing Tests
    // -------------------------------------------------------------------------

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
        assert!(app.word_char_results[0].iter().all(|r| *r == CharResult::Correct));
    }

    #[test]
    fn test_incorrect_typing() {
        let mut app = running_app();

        app.on_char('t');
        app.on_char('h');
        app.on_char('x');

        assert_eq!(app.typed_input, "thx");
        assert_eq!(app.correct_chars, 2);
        assert!(app.current_word_has_error);
        assert_eq!(app.word_char_results[0][2], CharResult::Incorrect);
    }

    #[test]
    fn test_space_advances_word() {
        let mut app = running_app();

        for c in "the".chars() {
            app.on_char(c);
        }
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

        for c in "thx".chars() {
            app.on_char(c);
        }
        app.on_space();

        assert_eq!(app.words_with_errors, vec![0]);
        assert!(!app.word_completed_correctly(0));
    }

    #[test]
    fn test_word_completed_correctly() {
        let mut app = running_app();

        for c in "the".chars() {
            app.on_char(c);
        }
        app.on_space();

        assert!(app.word_completed_correctly(0));
        assert!(!app.word_completed_correctly(1));
    }

    #[test]
    fn test_backspace() {
        let mut app = running_app();

        for c in "thx".chars() {
            app.on_char(c);
        }
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

    // -------------------------------------------------------------------------
    // Timer Tests
    // -------------------------------------------------------------------------

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

    // -------------------------------------------------------------------------
    // Stats Tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_stats_accuracy_all_correct() {
        let mut app = running_app();

        for c in "the".chars() {
            app.on_char(c);
        }

        let stats = app.calculate_stats();
        assert_eq!(stats.accuracy, 100.0);
        assert_eq!(stats.total_errors, 0);
    }

    #[test]
    fn test_stats_accuracy_with_errors() {
        let mut app = running_app();

        for c in "thxe".chars() {
            app.on_char(c);
        }

        let stats = app.calculate_stats();
        assert_eq!(stats.correct_chars, 2);
        assert_eq!(stats.total_chars_typed, 4);
        assert_eq!(stats.accuracy, 50.0);
        assert_eq!(stats.total_errors, 2);
    }

    #[test]
    fn test_stats_words_with_errors() {
        let mut app = running_app();

        for c in "the".chars() {
            app.on_char(c);
        }
        app.on_space();

        for c in "qxick".chars() {
            app.on_char(c);
        }
        app.on_space();

        let stats = app.calculate_stats();
        assert_eq!(stats.words_with_errors, 1);
    }

    // -------------------------------------------------------------------------
    // State Transition Tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_reset_returns_to_menu() {
        let mut app = running_app();

        for c in "the".chars() {
            app.on_char(c);
        }
        app.on_space();
        app.reset();

        assert_eq!(app.state, AppState::Menu);
        assert_eq!(app.current_word_idx, 0);
        assert!(app.typed_input.is_empty());
        assert_eq!(app.total_chars_typed, 0);
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

        for word in ["the", "quick", "brown"] {
            for c in word.chars() {
                app.on_char(c);
            }
            app.on_space();
        }

        assert_eq!(app.state, AppState::Finished);
    }

    #[test]
    fn test_time_mode_does_not_finish_on_last_word() {
        let mut app = App::with_words(test_words());
        app.test_mode = TestMode::Time;

        for word in ["the", "quick", "brown"] {
            for c in word.chars() {
                app.on_char(c);
            }
            app.on_space();
        }

        assert_eq!(app.state, AppState::Running);
    }
}
