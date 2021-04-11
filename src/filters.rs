/**
 * The filters module contains all the simple little functions for filtering english text into
 * usable tokens for the search index
 */

const STOPWORDS: &'static [&'static str] = &[
    "the",
    "be",
    "to",
    "of",
    "and",
    "a",
    "in",
    "that",
    "have",
    "i",
    "it",
    "for",
    "not",
    "on",
    "with",
    "he",
    "as",
    "you",
    "do",
    "at",
    "this",
    "but",
    "his",
    "by",
    "from",
    "wikipedia",
];

pub fn filter(text: &str) -> Vec<String> {
    use rust_stemmers::{Algorithm, Stemmer};

    let mut stems = vec![];
    // Create a stemmer for the english language
    let en_stemmer = Stemmer::create(Algorithm::English);

    for token in text.split(' ') {
        let token: String = token.to_lowercase()
            .chars()
            .filter(|ch| ! ch.is_ascii_punctuation())
            .collect();

        // only stem the word if it doesn't match a stopword
        if ! STOPWORDS.contains(&token.as_str()) {
            stems.push(en_stemmer.stem(&token).to_string());
        }
    }
    stems
}

#[cfg(test)]
mod tests {
    use super::*;
}
