mod adac;
pub use adac::*;

mod radio7;
pub use radio7::*;
use scraper::{CongestionDirection, CongestionScraper, CongestionScraperDatabase};

pub fn make_hagnau_scraper(db: impl Into<CongestionScraperDatabase>) -> CongestionScraper {
    let adac_b31 = ADACSource::new("D", "BW", ADACStreet::federal_road("B31"))
        .with_filters(ADACFilters::default().any_section_contains("hagnau"))
        .with_direction_classifier(|item| {
            if item
                .description
                .to_lowercase()
                .contains("richtung friedrichshafen")
            {
                CongestionDirection::Outbound
            } else {
                CongestionDirection::Inbound
            }
        });

    let radio7_b31 = Radio7Source::new()
        .with_filters(Radio7Filters::default().road_name("B31").title("hagnau"))
        .with_direction_classifier(|item| {
            if item
                .title
                .to_lowercase()
                .contains("richtung friedrichshafen")
            {
                CongestionDirection::Outbound
            } else {
                CongestionDirection::Inbound
            }
        });

    CongestionScraper::new(db)
        .with_source(radio7_b31)
        .with_source(adac_b31)
}
