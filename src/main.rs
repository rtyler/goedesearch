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
    #[options(help="A string to query for")]
    query: Option<String>,
}

impl CLI {
    fn query(index: &Index, query: &str) {
        println!("Querying for: `{}`", query);
        let documents = index.query_index(query);
        println!("Found {} documents", documents.len());
        for id in documents {
            if let Some(document) = index.document(&id) {
                println!("{}\n-------------------", document);
            }
        }
    }
}

fn main() -> Result<(), std::io::Error>{
    use rustyline::Editor;
    use rustyline::error::ReadlineError;

    pretty_env_logger::init();
    let opts = CLI::parse_args_or_exit(gumdrop::ParsingStyle::AllOptions);
    println!("Loading data file: {:?}", opts.datafile);

    let entries = Article::load_from_file(&opts.datafile)?;
    println!("Parsed {} entries", entries.len());

    let mut index = Index::new();
    for entry in entries {
        index.index_document(entry)?;
    }

    if let Some(query) = &opts.query {
        CLI::query(&index, query);
    }
    else {
        let history = ".geodesearch-history.txt";
        let mut rl = Editor::<()>::new();

        if rl.load_history(history).is_err() {
            info!("No previous history.");
        }
        loop {
            match rl.readline("query> ") {
                Ok(line) => {
                    CLI::query(&index, &line);
                },
                Err(ReadlineError::Eof) => { break },
                Err(ReadlineError::Interrupted) => { break },
                Err(err) => {
                    error!("Failed while reading line: {:?}", err);
                    break
                },
            }
        }

        rl.save_history(history).expect("Failed to save history");
    }

    Ok(())
}

type DocumentId = u64;
/**
 * A search index
 */
#[derive(Clone, Debug)]
struct Index {
    /**
     * Global mapping of each document and its id
     */
    documents: HashMap<DocumentId, Article>,
    /**
     * The frequencies of a term in the given document, keyed by the DocumentId
     * and the term within the document.
     */
    freq: HashMap<(DocumentId, String), f64>,
    /**
     * Index containing a mapping of terms to the documents which refer to them
     */
    index: HashMap<String, HashSet<DocumentId>>,
}

impl Index {
    fn new() -> Self {
        Self {
            documents: HashMap::default(),
            index: HashMap::default(),
            freq: HashMap::default(),
        }
    }

    /**
     * Attempt to retrieve the given document from the index
     */
    fn document(&self, id: &DocumentId) -> Option<&Article> {
        self.documents.get(id)
    }

    /**
     * Query the index for the given query string
     *
     * The query will be normalized and an ordering of document IDs will be returned
     */
    fn query_index(&self, query: &str) -> Vec<DocumentId> {
        let normalized = filters::filter(query);
        let mut sets = vec![];

        for token in normalized.iter() {
            if let Some(doc_ids) = self.index.get(token) {
                debug!("Docs found for token `{}`: {:?}", token, doc_ids);
                sets.push(doc_ids);
            }
        }

        // Depending on how mnay sets were collected, return the intersection
        let documents = match sets.len() {
            0 => HashSet::new(),
            _ => {
                sets[0]
                    .iter()
                    .filter(|b| sets[1..].iter().all(|set| set.contains(*b)))
                    .map(|b| *b)
                    .collect()
            }
        };

        /*
         * Time to rank these documents based on query
         */
        let mut results = vec![];
        let total_docs = self.documents.len() as f64;

        for id in documents.iter() {
            let mut score = 0.0;

            for token in normalized.iter() {
                if let Some(term_frequency) = self.freq.get(&(*id, token.to_string())) {
                    // inverse document frequency
                    let idf = ((total_docs / term_frequency) as f64).log10();
                    score += idf * term_frequency;
                }
            }

            debug!("Doc: {} has score: {}", id, score);
            results.push((id, score));
        }

        /*
         * Sort the results by whoever has the highest score and return
         */
        results.sort_by(|a,b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Less));
        debug!("Document scores: {:?}", results);
        results.iter().map(|r| *r.0).collect()
    }

    fn index_document(&mut self, article: Article) -> Result<(), std::io::Error> {
        let id = article.id();
        if ! self.documents.contains_key(&id) {
            let tokens = crate::filters::filter(&article.fulltext());

            // Make sure we have each token from the document in the index
            for token in tokens.iter() {
                // TODO: Find a way around this clone
                let freq_tuple = (id, token.clone());
                if ! self.freq.contains_key(&freq_tuple) {
                    self.freq.insert(freq_tuple, 1.0);
                }
                else {
                    if let Some(freq) = self.freq.get_mut(&freq_tuple) {
                        *freq += 1.0;
                    }
                }

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
        }
        Ok(())
    }
}

/**
 * A simple container struct to read the wikipedia dump
 */
#[derive(Clone, Debug, Deserialize)]
struct Feed {
    doc: Vec<Article>,
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

impl std::fmt::Display for Article {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(f, "{}\t({})\n    {}\n<{}>", self.title, self.id(), self.r#abstract, self.url)
    }
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
