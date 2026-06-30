use scraper::CongestionSource;

#[derive(Default)]
pub struct ADACFilters {
    federal_state: Option<String>,
    street: Option<String>,
}

impl ADACFilters {
    pub fn federal_state(mut self, federal_state: impl Into<String>) -> Self {
        self.federal_state = Some(federal_state.into());
        self
    }

    pub fn street(mut self, street: impl Into<String>) -> Self {
        self.street = Some(street.into());
        self
    }
}

/// ADAC congestion source
///
/// Example request:
/// https://www.adac.de/bff/?operationName=TrafficNews&variables=%7B%22filter%22%3A%7B%22country%22%3A%22D%22%2C%22federalState%22%3A%22BW%22%2C%22street%22%3A%22B31%22%2C%22showConstructionSites%22%3Afalse%2C%22pageNumber%22%3A1%7D%7D&extensions=%7B%22persistedQuery%22%3A%7B%22version%22%3A1%2C%22sha256Hash%22%3A%221166f30b5a011b4a848c0d75f78a64f7b5e1f721ca71498b099cd1173597888f%22%7D%7D
pub struct ADACSource {
    filters: ADACFilters,
}

impl CongestionSource for ADACSource {
    fn source_id(&self) -> &'static str {
        "adac"
    }

    fn poll(&self) -> Option<scraper::CongestionAmount> {
        None
    }
}
