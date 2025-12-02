//! Word generation for the typing test.
//!
//! This module provides a pool of common English words and a function
//! to generate random word lists for typing tests.

use rand::seq::SliceRandom;
use rand::thread_rng;

/// Pool of common English words used in typing tests.
///
/// Words are selected from the most common English words to ensure
/// familiarity and reasonable typing difficulty.
const WORDS: &[&str] = &[
    "the", "be", "to", "of", "and", "a", "in", "that", "have", "it",
    "for", "not", "on", "with", "he", "as", "you", "do", "at", "this",
    "but", "his", "by", "from", "they", "we", "say", "her", "she", "or",
    "an", "will", "my", "one", "all", "would", "there", "their", "what", "so",
    "up", "out", "if", "about", "who", "get", "which", "go", "me", "when",
    "make", "can", "like", "time", "no", "just", "him", "know", "take", "people",
    "into", "year", "your", "good", "some", "could", "them", "see", "other", "than",
    "then", "now", "look", "only", "come", "its", "over", "think", "also", "back",
    "after", "use", "two", "how", "our", "work", "first", "well", "way", "even",
    "new", "want", "because", "any", "these", "give", "day", "most", "us", "is",
    "water", "long", "find", "here", "thing", "great", "through", "world", "need", "much",
    "right", "still", "own", "try", "tell", "too", "high", "such", "off", "hand",
    "why", "while", "last", "might", "never", "down", "should", "small", "every", "home",
    "where", "move", "live", "same", "feel", "seem", "begin", "since", "part", "place",
    "made", "old", "big", "leave", "put", "end", "does", "another", "went", "been",
    "call", "few", "very", "run", "more", "write", "set", "change", "play", "must",
    "ask", "next", "stop", "keep", "start", "show", "hear", "city", "point", "turn",
    "name", "read", "help", "line", "each", "house", "both", "side", "group", "under",
    "word", "sound", "open", "state", "late", "eye", "head", "study", "public", "night",
    "child", "fact", "young", "early", "life", "face", "stand", "page", "paper", "watch",
];

/// Generates a random list of words for a typing test.
///
/// Words are randomly selected from the word pool with replacement,
/// meaning the same word may appear multiple times in the list.
///
/// # Arguments
///
/// * `count` - The number of words to generate
///
/// # Returns
///
/// A vector of randomly selected words
///
/// # Example
///
/// ```
/// let words = generate_words(25);
/// assert_eq!(words.len(), 25);
/// ```
pub fn generate_words(count: usize) -> Vec<String> {
    let mut rng = thread_rng();
    let mut words: Vec<String> = Vec::with_capacity(count);

    for _ in 0..count {
        if let Some(word) = WORDS.choose(&mut rng) {
            words.push(word.to_string());
        }
    }

    words
}
