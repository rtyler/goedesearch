/*
 * Goedesearch is an implementation of Bart's full text search engine as an exercise in Rust
 *
 * To learn more about it in Python: https://bart.degoe.de/building-a-full-text-search-engine-150-lines-of-code/
 */

use chrono::prelude::*;
use gumdrop::Options;
use log::*;
use std::path::PathBuf;

mod engine;
mod filters;

#[derive(Debug, Options)]
struct CLI {
    #[options(help = "print help message")]
    help: bool,
    #[options(required, help = "Specify the data file")]
    datafile: PathBuf,
    #[options(help = "A string to query for")]
    query: Option<String>,
}

impl CLI {
    fn query(index: &engine::Index, query: &str) {
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

fn main() -> Result<(), std::io::Error> {
    use rustyline::error::ReadlineError;
    use rustyline::Editor;

    pretty_env_logger::init();
    let opts = CLI::parse_args_or_exit(gumdrop::ParsingStyle::AllOptions);
    println!("Loading data file: {:?}", opts.datafile);

    let start = Utc::now();
    let index = engine::Index::from_file(&opts.datafile)?;
    println!("Parsed and indexed {} entries", index.size());
    println!(">> took {}s", (Utc::now() - start));

    if let Some(query) = &opts.query {
        CLI::query(&index, query);
    } else {
        let history = ".geodesearch-history.txt";
        let mut rl = Editor::<()>::new();

        if rl.load_history(history).is_err() {
            info!("No previous history.");
        }
        loop {
            match rl.readline("query> ") {
                Ok(line) => {
                    let start = Utc::now();
                    CLI::query(&index, &line);
                    println!(">> took {}s", (Utc::now() - start));
                }
                Err(ReadlineError::Eof) => break,
                Err(ReadlineError::Interrupted) => break,
                Err(err) => {
                    error!("Failed while reading line: {:?}", err);
                    break;
                }
            }
        }

        rl.save_history(history).expect("Failed to save history");
    }

    Ok(())
}
