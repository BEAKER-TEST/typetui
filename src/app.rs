use std::time::{Duration, Instant};

use crate::words::generate_words;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AppState {
    Waiting,
    Running,
    Finished,
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
}

impl App {
    pub fn new() -> Self {
        Self::with_words(generate_words(100))
    }

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
            state: AppState::Waiting,
        }
    }

    pub fn reset(&mut self) {
        let words = generate_words(100);
        let word_char_results = vec![Vec::new(); words.len()];

        self.words = words;
        self.current_word_idx = 0;
        self.typed_input = String::new();
        self.word_char_results = word_char_results;
        self.current_word_has_error = false;
        self.words_with_errors = Vec::new();
        self.total_chars_typed = 0;
        self.correct_chars = 0;
        self.start_time = None;
        self.state = AppState::Waiting;
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
        if self.state == AppState::Running {
            if let Some(start) = self.start_time {
                if start.elapsed() >= self.duration {
                    self.state = AppState::Finished;
                }
            }
        }
    }

    pub fn on_char(&mut self, c: char) {
        if self.state == AppState::Finished {
            return;
        }

        if self.state == AppState::Waiting {
            self.state = AppState::Running;
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
        if self.state == AppState::Finished {
            return;
        }

        if self.state == AppState::Waiting {
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
    }

    pub fn calculate_stats(&self) -> Stats {
        let elapsed_secs = match self.start_time {
            Some(start) => {
                let elapsed = start.elapsed();
                if elapsed > self.duration {
                    self.duration.as_secs_f64()
                } else {
                    elapsed.as_secs_f64()
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

    #[test]
    fn test_initial_state() {
        let app = App::with_words(test_words());

        assert_eq!(app.state, AppState::Waiting);
        assert_eq!(app.current_word_idx, 0);
        assert!(app.typed_input.is_empty());
        assert_eq!(app.total_chars_typed, 0);
        assert_eq!(app.correct_chars, 0);
        assert!(app.start_time.is_none());
    }

    #[test]
    fn test_first_char_starts_timer() {
        let mut app = App::with_words(test_words());

        assert_eq!(app.state, AppState::Waiting);
        assert!(app.start_time.is_none());

        app.on_char('t');

        assert_eq!(app.state, AppState::Running);
        assert!(app.start_time.is_some());
    }

    #[test]
    fn test_correct_typing() {
        let mut app = App::with_words(test_words());

        // Type "the" correctly
        app.on_char('t');
        app.on_char('h');
        app.on_char('e');

        assert_eq!(app.typed_input, "the");
        assert_eq!(app.total_chars_typed, 3);
        assert_eq!(app.correct_chars, 3);
        assert!(!app.current_word_has_error);

        // All results should be correct
        assert_eq!(app.word_char_results[0].len(), 3);
        assert!(app.word_char_results[0].iter().all(|r| *r == CharResult::Correct));
    }

    #[test]
    fn test_incorrect_typing() {
        let mut app = App::with_words(test_words());

        // Type "thx" instead of "the"
        app.on_char('t');
        app.on_char('h');
        app.on_char('x'); // wrong

        assert_eq!(app.typed_input, "thx");
        assert_eq!(app.total_chars_typed, 3);
        assert_eq!(app.correct_chars, 2);
        assert!(app.current_word_has_error);

        assert_eq!(app.word_char_results[0][2], CharResult::Incorrect);
    }

    #[test]
    fn test_space_advances_word() {
        let mut app = App::with_words(test_words());

        // Type "the" and press space
        app.on_char('t');
        app.on_char('h');
        app.on_char('e');
        app.on_space();

        assert_eq!(app.current_word_idx, 1);
        assert!(app.typed_input.is_empty());
        assert!(!app.current_word_has_error);
    }

    #[test]
    fn test_space_does_not_start_timer() {
        let mut app = App::with_words(test_words());

        app.on_space();

        assert_eq!(app.state, AppState::Waiting);
        assert!(app.start_time.is_none());
        assert_eq!(app.current_word_idx, 0);
    }

    #[test]
    fn test_word_with_error_tracked() {
        let mut app = App::with_words(test_words());

        // Type "thx" (wrong) and press space
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
        let mut app = App::with_words(test_words());

        // Type "the" correctly and press space
        app.on_char('t');
        app.on_char('h');
        app.on_char('e');
        app.on_space();

        assert!(app.word_completed_correctly(0));
        assert!(!app.word_completed_correctly(1)); // not completed yet
    }

    #[test]
    fn test_backspace() {
        let mut app = App::with_words(test_words());

        app.on_char('t');
        app.on_char('h');
        app.on_char('x'); // wrong

        assert_eq!(app.typed_input, "thx");
        assert_eq!(app.word_char_results[0].len(), 3);

        app.on_backspace();

        assert_eq!(app.typed_input, "th");
        assert_eq!(app.word_char_results[0].len(), 2);
    }

    #[test]
    fn test_backspace_on_empty_does_nothing() {
        let mut app = App::with_words(test_words());
        app.on_char('t'); // start the timer first
        app.on_backspace();
        app.on_backspace(); // should not panic or do anything weird

        assert!(app.typed_input.is_empty());
    }

    #[test]
    fn test_backspace_in_waiting_state_does_nothing() {
        let mut app = App::with_words(test_words());

        app.on_backspace();

        assert_eq!(app.state, AppState::Waiting);
    }

    #[test]
    fn test_time_remaining_before_start() {
        let app = App::with_words(test_words());

        assert_eq!(app.time_remaining(), Duration::from_secs(30));
    }

    #[test]
    fn test_time_remaining_after_start() {
        let mut app = App::with_words(test_words());
        app.on_char('t');

        // Time remaining should be less than 30 seconds
        assert!(app.time_remaining() <= Duration::from_secs(30));
        assert!(app.time_remaining() > Duration::from_secs(29));
    }

    #[test]
    fn test_stats_accuracy_all_correct() {
        let mut app = App::with_words(test_words());

        app.on_char('t');
        app.on_char('h');
        app.on_char('e');

        let stats = app.calculate_stats();
        assert_eq!(stats.accuracy, 100.0);
        assert_eq!(stats.total_errors, 0);
    }

    #[test]
    fn test_stats_accuracy_with_errors() {
        let mut app = App::with_words(test_words());

        app.on_char('t');
        app.on_char('h');
        app.on_char('x'); // wrong
        app.on_char('e'); // extra char, also wrong

        let stats = app.calculate_stats();
        assert_eq!(stats.correct_chars, 2);
        assert_eq!(stats.total_chars_typed, 4);
        assert_eq!(stats.accuracy, 50.0);
        assert_eq!(stats.total_errors, 2);
    }

    #[test]
    fn test_stats_words_with_errors() {
        let mut app = App::with_words(test_words());

        // First word correct
        app.on_char('t');
        app.on_char('h');
        app.on_char('e');
        app.on_space();

        // Second word with error
        app.on_char('q');
        app.on_char('x'); // wrong
        app.on_space();

        let stats = app.calculate_stats();
        assert_eq!(stats.words_with_errors, 1);
    }

    #[test]
    fn test_reset() {
        let mut app = App::with_words(test_words());

        app.on_char('t');
        app.on_char('h');
        app.on_char('e');
        app.on_space();

        app.reset();

        assert_eq!(app.state, AppState::Waiting);
        assert_eq!(app.current_word_idx, 0);
        assert!(app.typed_input.is_empty());
        assert_eq!(app.total_chars_typed, 0);
        assert_eq!(app.correct_chars, 0);
        assert!(app.start_time.is_none());
        assert!(app.words_with_errors.is_empty());
    }

    #[test]
    fn test_typing_after_finished_does_nothing() {
        let mut app = App::with_words(test_words());
        app.state = AppState::Finished;

        app.on_char('t');

        assert!(app.typed_input.is_empty());
        assert_eq!(app.total_chars_typed, 0);
    }

    #[test]
    fn test_space_after_finished_does_nothing() {
        let mut app = App::with_words(test_words());
        app.on_char('t'); // start
        app.state = AppState::Finished;

        let idx_before = app.current_word_idx;
        app.on_space();

        assert_eq!(app.current_word_idx, idx_before);
    }

    #[test]
    fn test_multiple_words_typed() {
        let mut app = App::with_words(test_words());

        // "the"
        for c in "the".chars() {
            app.on_char(c);
        }
        app.on_space();

        // "quick"
        for c in "quick".chars() {
            app.on_char(c);
        }
        app.on_space();

        assert_eq!(app.current_word_idx, 2);
        assert_eq!(app.correct_chars, 8); // "the" (3) + "quick" (5)
        assert!(app.word_completed_correctly(0));
        assert!(app.word_completed_correctly(1));
    }
}
