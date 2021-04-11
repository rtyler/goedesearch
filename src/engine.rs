/**
 * The engine module contains the bulk of the actual goedesearch engine
 */
use flate2::read::GzDecoder;
use log::*;
use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::path::PathBuf;
use url::Url;

/**
 * Alias to make sure that everything is using the same type for document IDs
 */
type DocumentId = u64;

/**
 * A wikipedia abstract data structure
 */
#[derive(Clone, Debug)]
pub struct Article {
    id: Option<DocumentId>,
    title: String,
    r#abstract: String,
    url: Option<Url>,
}

impl Default for Article {
    fn default() -> Self {
        Self {
            id: None,
            title: String::new(),
            r#abstract: String::new(),
            url: None,
        }
    }
}

impl std::fmt::Display for Article {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(
            f,
            "{}\t({})\n    {}\n<>",
            self.title,
            self.id(),
            self.r#abstract
        )
    }
}

impl Article {
    /**
     * Return the unique integer ID for the Article computed from the url
     */
    fn id(&self) -> DocumentId {
        self.id.unwrap_or(0)
    }

    fn set_url(&mut self, url: &str) -> Result<(), url::ParseError> {
        use crc::{crc64, Hasher64};

        let url = Url::parse(url).unwrap();

        let mut digest = crc64::Digest::new(crc64::ECMA);
        digest.write(url.as_str().as_bytes());
        self.url = Some(url);
        self.id = Some(digest.sum64());
        Ok(())
    }

    /**
     * Return the full text for the abstract which is basically just the
     * title and the brief description
     */
    fn fulltext(&self) -> String {
        format!("{} {}", self.title, self.r#abstract)
    }
}

/**
 * A search index
 */
#[derive(Clone, Debug)]
pub struct Index {
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
     * Load a Wikipedia XML dump from a gzip file
     */
    pub fn from_file(path: &PathBuf) -> Result<Self, std::io::Error> {
        use quick_xml::events::Event;
        use quick_xml::Reader;
        use std::io::BufReader;

        let mut index = Self::new();
        let file = File::open(path)?;
        let gz = GzDecoder::new(BufReader::new(file));
        let mut reader = Reader::from_reader(BufReader::new(gz));

        let mut buf = vec![];
        let mut article = None;

        loop {
            match reader.read_event(&mut buf) {
                // for triggering namespaced events, use this instead:
                // match reader.read_namespaced_event(&mut buf) {
                Ok(Event::Start(ref e)) => {
                    // for namespaced:
                    // Ok((ref namespace_value, Event::Start(ref e)))
                    match e.name() {
                        b"doc" => {
                            article = Some(Article::default());
                        }
                        b"title" => {
                            if let Some(ref mut article) = article {
                                article.title =
                                    reader.read_text(e.name(), &mut Vec::new()).unwrap();
                            }
                        }
                        b"abstract" => {
                            if let Some(ref mut article) = article {
                                article.r#abstract =
                                    reader.read_text(e.name(), &mut Vec::new()).unwrap();
                            }
                        }
                        b"url" => {
                            if let Some(ref mut article) = article {
                                let u = reader.read_text(e.name(), &mut Vec::new()).unwrap();
                                article.set_url(&u).unwrap();
                            }
                        }
                        _ => (),
                    }
                }
                Ok(Event::End(ref e)) => match e.name() {
                    b"doc" => {
                        index.index_document(article.unwrap()).unwrap();
                        article = None
                    }
                    _ => (),
                },
                // unescape and decode the text event using the reader encoding
                //Ok(Event::Text(e)) => txt.push(e.unescape_and_decode(&reader).unwrap()),
                Ok(Event::Eof) => break, // exits the loop when reaching end of file
                Err(e) => panic!("Error at position {}: {:?}", reader.buffer_position(), e),
                _ => (), // There are several other `Event`s we do not consider here
            }

            // if we don't keep a borrow elsewhere, we can clear the buffer to keep memory usage low
            buf.clear();
        }

        debug!("Found {} documents in the file", index.size());
        Ok(index)
    }

    /**
     * The number of documents in the index
     */
    pub fn size(&self) -> u64 {
        self.documents.len() as u64
    }

    /**
     * Attempt to retrieve the given document from the index
     */
    pub fn document(&self, id: &DocumentId) -> Option<&Article> {
        self.documents.get(id)
    }

    /**
     * Query the index for the given query string
     *
     * The query will be normalized and an ordering of document IDs will be returned
     */
    pub fn query_index(&self, query: &str) -> Vec<DocumentId> {
        let normalized = crate::filters::filter(query);
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
            _ => sets[0]
                .iter()
                .filter(|b| sets[1..].iter().all(|set| set.contains(*b)))
                .map(|b| *b)
                .collect(),
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
        results.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Less));
        debug!("Document scores: {:?}", results);
        results.iter().map(|r| *r.0).collect()
    }

    fn index_document(&mut self, article: Article) -> Result<(), std::io::Error> {
        let id = article.id();
        if !self.documents.contains_key(&id) {
            let tokens = crate::filters::filter(&article.fulltext());

            // Make sure we have each token from the document in the index
            for token in tokens.iter() {
                // TODO: Find a way around this clone
                let freq_tuple = (id, token.clone());
                if !self.freq.contains_key(&freq_tuple) {
                    self.freq.insert(freq_tuple, 1.0);
                } else {
                    if let Some(freq) = self.freq.get_mut(&freq_tuple) {
                        *freq += 1.0;
                    }
                }

                if !self.index.contains_key(token) {
                    self.index.insert(token.to_string(), HashSet::new());
                }
                if let Some(set) = self.index.get_mut(token) {
                    set.insert(id);
                } else {
                    warn!(
                        "Tried to get a mutable version of the index for {} and failed",
                        token
                    );
                }
            }

            self.documents.insert(id, article);
        }
        Ok(())
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
    fn test_index_simple_data() -> Result<(), std::io::Error> {
        let index = Index::from_file(&PathBuf::from("data/simple.xml.gz"))?;
        assert_eq!(index.size(), 356);
        Ok(())
    }

    #[test]
    fn test_index_size() {
        let index = Index::new();
        assert_eq!(index.size(), 0);
    }
}
