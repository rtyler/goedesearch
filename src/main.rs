/*
 * Goedesearch is an implementation of Bart's full text search engine as an exercise in Rust
 * 
 * To learn more about it in Python: https://bart.degoe.de/building-a-full-text-search-engine-150-lines-of-code/
 */

use flate2::read::GzDecoder;
use serde::Deserialize;
use std::fs::File;
use std::path::PathBuf;

fn main() -> Result<(), std::io::Error>{
    println!("Loading data file..");
    let entries = Article::load_from_file(&PathBuf::from("data/enwiki-latest-abstract.xml.gz"))?;
    println!("Parsed {} entries", entries.len());
    Ok(())
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
