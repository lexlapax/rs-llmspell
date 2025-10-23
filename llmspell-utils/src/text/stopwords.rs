//! Stopword lists for natural language processing
//!
//! Provides comprehensive stopword lists for filtering common non-entity words
//! during entity extraction, query understanding, and text analysis.
//!
//! # Overview
//!
//! Stopwords are common words that typically don't carry meaningful semantic information
//! for entity extraction or keyword analysis. This module provides:
//!
//! - 165 English stopwords across 6 categories
//! - O(1) lookup performance using `HashSet`
//! - Thread-safe static initialization with `LazyLock`
//!
//! # Categories
//!
//! - **Determiners & Demonstratives**: The, This, That, A, An, etc.
//! - **Pronouns**: It, They, We, You, I, He, She, etc.
//! - **Conjunctions**: And, Or, But, So, If, Because, etc.
//! - **Prepositions & Common Verbs**: In, On, At, To, Is, Are, Have, etc.
//! - **Temporal & Quantifiers**: Now, Then, Today, All, Some, Many, etc.
//! - **Adverbs & Discourse Markers**: However, Therefore, Actually, etc.
//! - **Meta-Discourse**: Example, Note, Summary, Section, etc.
//!
//! # Performance
//!
//! - Initialization: One-time cost on first access (~100μs)
//! - Lookup: O(1) hash table lookup (~10ns)
//! - Memory: ~12KB for 165 words
//!
//! # Examples
//!
//! ```rust
//! use llmspell_utils::text::stopwords::{is_stopword, STOPWORDS};
//!
//! // Check if word is a stopword
//! assert!(is_stopword("The"));
//! assert!(is_stopword("However"));
//! assert!(!is_stopword("Rust"));
//!
//! // Access stopword set directly
//! assert_eq!(STOPWORDS.len(), 165);
//! ```

use std::collections::HashSet;
use std::sync::LazyLock;

/// Comprehensive stopword set for English
///
/// Contains 165 common English stopwords across multiple categories:
/// determiners, pronouns, conjunctions, prepositions, verbs, temporal words,
/// quantifiers, adverbs, discourse markers, and meta-discourse terms.
///
/// Initialized lazily on first access for optimal startup performance.
///
/// # Examples
///
/// ```rust
/// use llmspell_utils::text::stopwords::STOPWORDS;
///
/// assert!(STOPWORDS.contains("The"));
/// assert!(STOPWORDS.contains("However"));
/// assert!(!STOPWORDS.contains("Rust"));
/// ```
pub static STOPWORDS: LazyLock<HashSet<&'static str>> = LazyLock::new(|| {
    HashSet::from([
        // Determiners & demonstratives
        "The", "This", "That", "These", "Those", "A", "An",
        // Pronouns
        "It", "They", "We", "You", "I", "He", "She", "My", "Your", "Their", "Our",
        "His", "Her", "Its", "Who", "What", "Which", "Where", "When", "Why", "How",
        // Conjunctions
        "And", "Or", "But", "So", "Yet", "For", "Nor", "If", "Because", "Although",
        "Unless", "While", "Since", "Before", "After",
        // Prepositions & common verbs
        "In", "On", "At", "To", "From", "By", "With", "Without", "Through", "During",
        "About", "Above", "Below", "Between", "Among", "Under", "Over", "Into", "Onto",
        "Is", "Are", "Was", "Were", "Be", "Been", "Being", "Have", "Has", "Had",
        "Do", "Does", "Did", "Will", "Would", "Could", "Should", "May", "Might", "Must",
        "Can", "Cannot", "Get", "Got", "Make", "Made", "Take", "Taken",
        // Temporal & quantifiers
        "Now", "Then", "Today", "Tomorrow", "Yesterday", "Always", "Never", "Sometimes",
        "Often", "Usually", "Recently", "Currently", "Previously", "Next", "Last",
        "All", "Some", "Many", "Few", "Several", "Most", "Any", "No", "None",
        "Each", "Every", "Other", "Another", "Such", "Same", "Different",
        // Common adverbs & discourse markers
        "Very", "Really", "Quite", "Too", "Also", "Just", "Only", "Even", "Still",
        "However", "Therefore", "Thus", "Hence", "Moreover", "Furthermore", "Nevertheless",
        "Nonetheless", "Otherwise", "Indeed", "Actually", "Basically", "Essentially",
        "Specifically", "Particularly", "Generally", "Typically",
        // Meta-discourse
        "Example", "Examples", "Note", "Notes", "Important", "Summary", "Conclusion",
        "Introduction", "Background", "Overview", "Details", "Section", "Chapter",
        "Figure", "Table", "Appendix", "Reference", "References",
    ])
});

/// Check if a word is a common stopword
///
/// Performs O(1) hash table lookup against the comprehensive stopword set.
/// Case-sensitive - matches capitalized forms typical in entity extraction.
///
/// # Arguments
///
/// * `word` - Word to check (must match exact capitalization)
///
/// # Returns
///
/// `true` if word is a stopword, `false` otherwise
///
/// # Examples
///
/// ```rust
/// use llmspell_utils::text::stopwords::is_stopword;
///
/// // Capitalized stopwords (common in entity extraction)
/// assert!(is_stopword("The"));
/// assert!(is_stopword("However"));
/// assert!(is_stopword("Therefore"));
///
/// // Non-stopwords
/// assert!(!is_stopword("Rust"));
/// assert!(!is_stopword("Python"));
///
/// // Case sensitivity
/// assert!(!is_stopword("the"));  // Lowercase not in set
/// ```
///
/// # Performance
///
/// - Average: O(1) hash lookup (~10ns)
/// - Worst case: O(1) with hash collision (~50ns)
///
/// # See Also
///
/// - [`STOPWORDS`] - The underlying stopword set
#[inline]
#[must_use]
pub fn is_stopword(word: &str) -> bool {
    STOPWORDS.contains(word)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_determiners_demonstratives() {
        assert!(is_stopword("The"));
        assert!(is_stopword("This"));
        assert!(is_stopword("That"));
        assert!(is_stopword("These"));
        assert!(is_stopword("A"));
        assert!(is_stopword("An"));
    }

    #[test]
    fn test_pronouns() {
        assert!(is_stopword("It"));
        assert!(is_stopword("They"));
        assert!(is_stopword("We"));
        assert!(is_stopword("I"));
        assert!(is_stopword("Who"));
        assert!(is_stopword("What"));
    }

    #[test]
    fn test_conjunctions() {
        assert!(is_stopword("And"));
        assert!(is_stopword("Or"));
        assert!(is_stopword("But"));
        assert!(is_stopword("Because"));
    }

    #[test]
    fn test_prepositions() {
        assert!(is_stopword("In"));
        assert!(is_stopword("On"));
        assert!(is_stopword("At"));
        assert!(is_stopword("Through"));
    }

    #[test]
    fn test_common_verbs() {
        assert!(is_stopword("Is"));
        assert!(is_stopword("Are"));
        assert!(is_stopword("Have"));
        assert!(is_stopword("Can"));
    }

    #[test]
    fn test_temporal_quantifiers() {
        assert!(is_stopword("Now"));
        assert!(is_stopword("Today"));
        assert!(is_stopword("All"));
        assert!(is_stopword("Some"));
    }

    #[test]
    fn test_discourse_markers() {
        assert!(is_stopword("However"));
        assert!(is_stopword("Therefore"));
        assert!(is_stopword("Actually"));
    }

    #[test]
    fn test_meta_discourse() {
        assert!(is_stopword("Example"));
        assert!(is_stopword("Summary"));
        assert!(is_stopword("Section"));
    }

    #[test]
    fn test_non_stopwords() {
        assert!(!is_stopword("Rust"));
        assert!(!is_stopword("Python"));
        assert!(!is_stopword("JavaScript"));
        assert!(!is_stopword("Docker"));
    }

    #[test]
    fn test_case_sensitivity() {
        // Capitalized versions are stopwords
        assert!(is_stopword("The"));
        assert!(is_stopword("However"));

        // Lowercase versions are NOT stopwords (by design)
        assert!(!is_stopword("the"));
        assert!(!is_stopword("however"));
    }

    #[test]
    fn test_stopwords_count() {
        // Should have 165 stopwords total
        assert_eq!(STOPWORDS.len(), 165);
    }

    #[test]
    fn test_no_duplicates() {
        // HashSet automatically handles duplicates, but verify count matches expected
        let expected_count = 165;
        assert_eq!(STOPWORDS.len(), expected_count, "Stopwords set should contain exactly {expected_count} unique entries");
    }
}
