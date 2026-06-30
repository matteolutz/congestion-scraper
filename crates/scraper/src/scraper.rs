use std::path::PathBuf;

use crate::{CongestionEntry, CongestionSource};

#[derive(Clone)]
pub enum CongestionScraperDatabase {
    Memory,
    File(PathBuf),
}

impl<T: Into<PathBuf>> From<T> for CongestionScraperDatabase {
    fn from(path: T) -> Self {
        CongestionScraperDatabase::File(path.into())
    }
}

impl CongestionScraperDatabase {
    fn connect(self) -> rusqlite::Connection {
        match self {
            CongestionScraperDatabase::Memory => rusqlite::Connection::open_in_memory()
                .expect("[CongestionScraper] Failed to open SQLite connection"),
            CongestionScraperDatabase::File(path) => rusqlite::Connection::open(path)
                .expect("[CongestionScraper] Failed to open SQLite connection"),
        }
    }
}

pub struct CongestionScraper {
    sources: Vec<Box<dyn CongestionSource>>,

    sql: rusqlite::Connection,
}

impl CongestionScraper {
    pub fn new(db: impl Into<CongestionScraperDatabase>) -> Self {
        let sql = db.into().connect();
        CongestionEntry::ensure_table(&sql)
            .expect("[CongestionScraper] Failed to ensure congestion_entries table exists");

        Self {
            sources: Vec::new(),
            sql,
        }
    }

    pub fn with_source(mut self, source: impl CongestionSource + 'static) -> Self {
        self.sources.push(Box::new(source));
        self
    }

    pub fn start(self, polling_interval: impl Into<std::time::Duration>) {
        let polling_interval = polling_interval.into();

        loop {
            println!(
                "[CongestionScraper] Polling... Next poll in {:?}",
                polling_interval
            );
            self.poll();

            let count = CongestionEntry::count(&self.sql);
            println!(
                "[CongestionScraper] There are now {} entries in the database",
                count.unwrap_or(0)
            );

            std::thread::sleep(polling_interval);
        }
    }

    fn poll(&self) {
        let now = chrono::Utc::now();

        let entries = self.sources.iter().filter_map(|source| {
            let congestion_amount = source.poll()?;
            Some(CongestionEntry::new(
                now,
                source.source_id(),
                congestion_amount,
            ))
        });

        for entry in entries {
            let _ = entry
                .insert(&self.sql)
                .inspect_err(|err| println!("[CongestionScraper] Failed to insert entry: {}", err));
        }
    }
}
