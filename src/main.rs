/*
 * Goedesearch is an implementation of Bart's full text search engine as an exercise in Rust
 * 
 * To learn more about it in Python: https://bart.degoe.de/building-a-full-text-search-engine-150-lines-of-code/
 */

use flate2::read::GzDecoder;
use serde::Deserialize;
use std::fs::File;
use std::path::PathBuf;

fn main() {
    println!("Hello, world!");
}


/**
 * A wikipedia abstract data structure
 */
#[derive(Clone, Debug, Deserialize)]
#[serde(rename = "doc")]
struct Abstract {
    id: Option<u64>,
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
    doc: Vec<Abstract>,
}


impl Abstract {
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
        use quick_xml::Reader;
        use quick_xml::events::Event;

        let file = File::open(gzip_xml)?;
        let gz = GzDecoder::new(BufReader::new(file));
        println!("header: {:?}", gz.header());
        let mut reader = BufReader::new(gz);
        let mut line = String::new();

        use std::io::BufRead;
        let mut count = 0;
        loop {
            if count > 10 {
                break;
            }
            count += 1;
            let mut line = String::new();
            let len = reader.read_line(&mut line)?;
            println!("{} - {}", len, line);
        }
        return Ok(vec![]);

        let mut reader = Reader::from_reader(BufReader::new(gz));
        let mut buf = vec![];
        //let mut tags = std::collections::HashSet::new();

        loop {
            if count > 10 {
                break;
            }

            match reader.read_event(&mut buf) {
                Ok(Event::Start(ref e)) => {
                    let tag = e.name();
                    println!("tag: {}", String::from_utf8_lossy(tag));
                // for namespaced:
                // Ok((ref namespace_value, Event::Start(ref e)))
                    match tag {
                        b"doc" => count += 1,
                        _ => {
                           //tags.insert(String::from_utf8_lossy(&tag));
                        },
                    }
                },
                // unescape and decode the text event using the reader encoding
                //Ok(Event::Text(e)) => txt.push(e.unescape_and_decode(&reader).unwrap()),
                Ok(Event::Eof) => break, // exits the loop when reaching end of file
                Err(e) => panic!("Error at position {}: {:?}", reader.buffer_position(), e),
                other => {
                    println!("wtf: {:?}", other);
                }, // There are several other `Event`s we do not consider here
            }
        }
        println!("docs: {}", count);
        //let wikipedia: Feed = from_reader(BufReader::new(gz)).expect("Failed to read dump");

        Ok(vec![])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[ignore]
    #[test]
    fn test_simple_data() -> Result<(), std::io::Error> {
        let entries = Abstract::load_from_file(&PathBuf::from("simple.xml.gz"))?;
        assert!(entries.len() > 0);
        Ok(())
    }

    #[test]
    fn test_wikipedia_data() -> Result<(), std::io::Error> {
        let entries = Abstract::load_from_file(&PathBuf::from("enwiki-latest-abstract.xml.gz"))?;
        assert!(entries.len() > 0);
        Ok(())
    }
}
