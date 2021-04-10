/*
 * Goedesearch is an implementation of Bart's full text search engine as an exercise in Rust
 * 
 * To learn more about it in Python: https://bart.degoe.de/building-a-full-text-search-engine-150-lines-of-code/
 */

use flate2::read::GzDecoder;
use serde::Deserialize;
use std::fs::File;
use std::path::PathBuf;
use std::collections::HashMap;

fn main() -> Result<(), std::io::Error>{
    println!("Loading data file..");
    let entries = Article::load_from_file(&PathBuf::from("data/enwiki-latest-abstract.xml.gz"))?;
    println!("Parsed {} entries", entries.len());
    Ok(())
}

/**
 * A search index
 */
#[derive(Clone, Debug)]
struct Index {
    items: HashMap<u64, Article>,
}

mod Filters {
    const STOPWORDS: &'static [&'static str] = &["the", "be", "to", "of", "and",
    "a", "in", "that", "have", "i", "it", "for", "not", "on", "with", "he", "as",
    "you", "do", "at", "this", "but", "his", "by", "from", "wikipedia"];

    fn stems(tokens: Vec<String>) -> Vec<String> {
        use rust_stemmers::{Algorithm, Stemmer};
        // Create a stemmer for the english language
        let en_stemmer = Stemmer::create(Algorithm::English);
        tokens.iter().map(|token| {
            en_stemmer.stem(token).to_string()
        }).collect()
    }

    fn tokenize(text: &str) -> Vec<&str> {
        text.split(' ').collect()
    }

    fn lowercase(tokens: Vec<&str>) -> Vec<String> {
        tokens.iter().map(|t| t.to_lowercase()).collect()
    }

    fn punctuation(tokens: Vec<String>) -> Vec<String> {
        tokens.iter().map(|token| {
            token.chars()
                .filter(|c| !c.is_ascii_punctuation())
                .collect()
        }).collect()
    }

    fn stopwords(tokens: Vec<String>) -> Vec<String> {
        tokens.into_iter().filter(|token| {
            ! STOPWORDS.contains(&token.as_str())
        }).collect()
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn test_tokenize() {
            let buf = "yo hello world";
            assert_eq!(
                vec!["yo", "hello", "world"],
                tokenize(buf));
        }

        #[test]
        fn test_lowercase() {
            let tokens = vec!["HellO", "WORLd"];
            assert_eq!(
                vec!["hello", "world"],
                lowercase(tokens));
        }

        #[test]
        fn test_punctuation() {
            let tokens = vec!["This,".to_string(), "isn't".to_string(), "great?".to_string()];
            assert_eq!(
                vec!["This", "isnt", "great"],
                punctuation(tokens));
        }

        #[test]
        fn test_stopwords() {
            let tokens = vec!["i".to_string(),
                                "am".to_string(),
                                "the".to_string(),
                                "walrus".to_string()];
            assert_eq!(vec!["am", "walrus"], stopwords(tokens));
        }

        #[test]
        fn test_stems() {
            let tokens = vec!["help".to_string(),
                                "fruitlessly".to_string(),
                                "fruitless".to_string()];
            assert_eq!(vec!["help", "fruitless", "fruitless"], stems(tokens));
        }
    }
}

impl Index {
    fn new() -> Self {
        Self {
            items: HashMap::default(),
        }
    }

    fn index_document(&mut self, article: Article) -> Result<(), std::io::Error> {
        let id = article.id();
        if ! self.items.contains_key(&id) {
            // Should index
        }

        //  do something with tokens
        Ok(())
    }
}

/**
 * A wikipedia abstract data structure
 */
#[derive(Clone, Debug, Deserialize)]
struct Article {
    title: String,
    r#abstract: String,
    url: url::Url,
    #[serde(skip_deserializing)]
    links: Vec<String>,
}

/**
 * A simple container struct to read the wikipedia dump
 */
#[derive(Clone, Debug, Deserialize)]
struct Feed {
    doc: Vec<Article>,
}

impl Article {
    /**
     * Return the unique integer ID for the Article computed from the url
     */
    fn id(&self) -> u64 {
        use crc::{crc64, Hasher64};
        let mut digest = crc64::Digest::new(crc64::ECMA);
        digest.write(self.url.as_str().as_bytes());
        digest.sum64()
    }

    /**
     * Return the full text for the abstract which is basically just the
     * title and the brief description
     */
    fn fulltext(&self) -> String {
        format!("{} {}", self.title, self.r#abstract)
    }

    /**
     * Load the abstract entries from the referenced file
     */
    fn load_from_file(gzip_xml: &PathBuf) -> Result<Vec<Self>, std::io::Error> {
        use std::io::BufReader;
        use quick_xml::de::from_reader;

        let file = File::open(gzip_xml)?;
        let gz = GzDecoder::new(BufReader::new(file));
        let wikipedia: Feed = from_reader(BufReader::new(gz)).expect("Failed to read dump");
        Ok(wikipedia.doc)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_data() -> Result<(), std::io::Error> {
        let entries = Article::load_from_file(&PathBuf::from("data/simple.xml.gz"))?;
        assert!(entries.len() > 0);
        Ok(())
    }
}
