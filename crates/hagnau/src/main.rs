use scraper::{CongestionDirection, CongestionScraper};

use crate::sources::{Radio7Filters, Radio7Source};

mod sources;

pub fn main() {
    // get db path from first argument
    let db_path = std::env::args()
        .nth(1)
        .expect("Please provide the path to the SQLite DB file as the first command line argument");

    let radio7_b31 = Radio7Source::new()
        .with_filters(Radio7Filters::default().road_name("B31").title("hagnau"))
        .with_direction_classifier(|traffic| {
            if traffic
                .title
                .to_lowercase()
                .contains("richtung friedrichshafen")
            {
                CongestionDirection::Outbound
            } else {
                CongestionDirection::Inbound
            }
        });

    let scraper = CongestionScraper::new(db_path).with_source(radio7_b31);
    scraper.start(std::time::Duration::from_mins(5));
}
