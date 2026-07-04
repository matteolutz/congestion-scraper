pub fn main() {
    // get db path from first argument
    let db_path = std::env::args()
        .nth(1)
        .expect("Please provide the path to the SQLite DB file as the first command line argument");

    let scraper = hagnau_sources::make_hagnau_scraper(db_path);

    scraper.start(std::time::Duration::from_mins(5));
}
