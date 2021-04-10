/*
 * Goedesearch is an implementation of Bart's full text search engine as an exercise in Rust
 * 
 * To learn more about it in Python: https://bart.degoe.de/building-a-full-text-search-engine-150-lines-of-code/
 */

use flate2::read::GzDecoder;
use gumdrop::Options;
use log::*;
use serde::Deserialize;
use std::fs::File;
use std::path::PathBuf;
use std::collections::{HashMap, HashSet};

mod filters;

#[derive(Debug, Options)]
struct CLI {
    #[options(help = "print help message")]
    help: bool,
    #[options(required, help="Specify the data file")]
    datafile: PathBuf,

    #[options(free, help="A string to query for")]
    query: String,
}

fn main() -> Result<(), std::io::Error>{
    let opts = CLI::parse_args_or_exit(gumdrop::ParsingStyle::AllOptions);
    println!("Loading data file: {:?}", opts.datafile);

    let entries = Article::load_from_file(&opts.datafile)?;
    println!("Parsed {} entries", entries.len());

    let mut index = Index::new();
    for entry in entries {
        index.index_document(entry)?;
    }
    println!("Querying for: `{}`", opts.query);
    let documents = index.query_index(&opts.query);
    println!("Found {} documents", documents.len());
    for id in documents {
        println!("{:?}", index.document(&id));
    }
    Ok(())
}

type DocumentId = u64;
/**
 * A search index
 */
#[derive(Clone, Debug)]
struct Index {
    documents: HashMap<DocumentId, Article>,
    index: HashMap<String, HashSet<DocumentId>>,
}

impl Index {
    fn new() -> Self {
        Self {
            documents: HashMap::default(),
            index: HashMap::default(),
        }
    }

    fn document(&self, id: &DocumentId) -> Option<&Article> {
        self.documents.get(id)
    }

    fn query_index(&self, query: &str) -> HashSet<DocumentId> {
        let normalized = filters::filter(query);
        let mut sets = vec![];

        for token in normalized {
            if let Some(doc_ids) = self.index.get(&token) {
                debug!("Docs found for token `{}`: {:?}", token, doc_ids);
                sets.push(doc_ids);
            }
        }

        // Depending on how mnay sets were collected, return the intersection
        match sets.len() {
            0 => HashSet::new(),
            //1 => sets.pop().unwrap(),
            _ => {
                sets[0]
                    .iter()
                    .filter(|b| sets[1..].iter().all(|set| set.contains(*b)))
                    .map(|b| *b)
                    .collect()
            }
        }
    }

    fn index_document(&mut self, article: Article) -> Result<(), std::io::Error> {
        let id = article.id();
        if ! self.documents.contains_key(&id) {
            let tokens = crate::filters::filter(&article.fulltext());

            // Make sure we have each token from the document in the index
            for token in tokens.iter() {
                if ! self.index.contains_key(token) {
                    self.index.insert(token.to_string(), HashSet::new());
                }
                if let Some(set) = self.index.get_mut(token) {
                    set.insert(id);
                }
                else {
                    warn!("Tried to get a mutable version of the index for {} and failed", token);
                }
            }

            self.documents.insert(id, article);
            // TODO: Should analyze which means a term frequency count
        }
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
    fn id(&self) -> DocumentId {
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
    fn test_index_new() {
        let _index = Index::new();
    }

    #[test]
    fn test_simple_data() -> Result<(), std::io::Error> {
        let entries = Article::load_from_file(&PathBuf::from("data/simple.xml.gz"))?;
        assert!(entries.len() > 0);
        Ok(())
    }
}
