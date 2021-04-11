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
    stems(stopwords(punctuation(lowercase(tokenize(text)))))
}

fn stems(tokens: Vec<String>) -> Vec<String> {
    use rust_stemmers::{Algorithm, Stemmer};
    // Create a stemmer for the english language
    let en_stemmer = Stemmer::create(Algorithm::English);
    tokens
        .iter()
        .map(|token| en_stemmer.stem(token).to_string())
        .collect()
}

fn tokenize(text: &str) -> Vec<&str> {
    text.split(' ').collect()
}

fn lowercase(tokens: Vec<&str>) -> Vec<String> {
    tokens.iter().map(|t| t.to_lowercase()).collect()
}

fn punctuation(tokens: Vec<String>) -> Vec<String> {
    tokens
        .iter()
        .map(|token| {
            token
                .chars()
                .filter(|c| !c.is_ascii_punctuation())
                .collect()
        })
        .collect()
}

fn stopwords(tokens: Vec<String>) -> Vec<String> {
    tokens
        .into_iter()
        .filter(|token| !STOPWORDS.contains(&token.as_str()))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tokenize() {
        let buf = "yo hello world";
        assert_eq!(vec!["yo", "hello", "world"], tokenize(buf));
    }

    #[test]
    fn test_lowercase() {
        let tokens = vec!["HellO", "WORLd"];
        assert_eq!(vec!["hello", "world"], lowercase(tokens));
    }

    #[test]
    fn test_punctuation() {
        let tokens = vec![
            "This,".to_string(),
            "isn't".to_string(),
            "great?".to_string(),
        ];
        assert_eq!(vec!["This", "isnt", "great"], punctuation(tokens));
    }

    #[test]
    fn test_stopwords() {
        let tokens = vec![
            "i".to_string(),
            "am".to_string(),
            "the".to_string(),
            "walrus".to_string(),
        ];
        assert_eq!(vec!["am", "walrus"], stopwords(tokens));
    }

    #[test]
    fn test_stems() {
        let tokens = vec![
            "help".to_string(),
            "fruitlessly".to_string(),
            "fruitless".to_string(),
        ];
        assert_eq!(vec!["help", "fruitless", "fruitless"], stems(tokens));
    }
}
