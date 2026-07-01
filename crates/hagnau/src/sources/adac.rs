use scraper::{CongestionDirection, CongestionSource};

#[derive(Debug)]
pub struct ADACNewsItem {
    #[allow(unused)]
    pub street: String,

    pub street_sections: Vec<String>,

    pub description: String,
    pub congestion_minutes: f64,
}

#[derive(Default)]
pub struct ADACFilters {
    any_section_contains_one_of: Vec<String>,
    description_contains_one_of: Vec<String>,
}

impl ADACFilters {
    pub fn any_section_contains(mut self, section: impl Into<String>) -> Self {
        self.any_section_contains_one_of.push(section.into());
        self
    }

    pub fn description_contains(mut self, description: impl Into<String>) -> Self {
        self.description_contains_one_of.push(description.into());
        self
    }
}

impl ADACFilters {
    fn apply(
        &self,
        items: impl Iterator<Item = ADACNewsItem>,
    ) -> impl Iterator<Item = ADACNewsItem> {
        items.filter(|item| {
            if !self.any_section_contains_one_of.is_empty()
                && !self
                    .any_section_contains_one_of
                    .iter()
                    .any(|filter_section| {
                        let filter_section = filter_section.to_lowercase();
                        item.street_sections
                            .iter()
                            .any(|section| section.to_lowercase().contains(&filter_section))
                    })
            {
                return false;
            }

            if !self.description_contains_one_of.is_empty()
                && !self
                    .description_contains_one_of
                    .iter()
                    .any(|filter_description| {
                        item.description
                            .to_lowercase()
                            .contains(&filter_description.to_lowercase())
                    })
            {
                return false;
            }

            true
        })
    }
}

#[derive(Debug)]
pub enum ADACStreet {
    FederalRoad(String),
    Highway(String),
}

impl ADACStreet {
    pub fn federal_road(name: impl Into<String>) -> Self {
        Self::FederalRoad(name.into())
    }

    pub fn highway(name: impl Into<String>) -> Self {
        Self::Highway(name.into())
    }

    fn street_name(&self) -> &str {
        match self {
            ADACStreet::FederalRoad(name) | ADACStreet::Highway(name) => name.as_str(),
        }
    }

    fn street_type(&self) -> &str {
        match self {
            ADACStreet::FederalRoad(_) => "FederalRoad",
            ADACStreet::Highway(_) => "Highway",
        }
    }
}

const PAGE_SIZE: usize = 10;

/// ADAC congestion source
///
/// Example request:
/// https://www.adac.de/verkehr/verkehrsinformationen/de/baden-wuerttemberg/?country=D&federalState=BW&street=B31&streetType=FederalRoad&showConstructionSites=false&pageNumber=1&submit=true&resetSearchParams=false
pub struct ADACSource {
    country: String,
    federal_state: String,
    street: ADACStreet,

    filters: ADACFilters,
    direction_classifier: Option<Box<dyn Fn(&ADACNewsItem) -> CongestionDirection>>,

    client: reqwest::blocking::Client,
}

impl ADACSource {
    pub fn new(
        country: impl Into<String>,
        federal_state: impl Into<String>,
        street: ADACStreet,
    ) -> Self {
        let client = reqwest::blocking::Client::new();

        Self {
            country: country.into(),
            federal_state: federal_state.into(),
            street: street.into(),
            filters: ADACFilters::default(),
            direction_classifier: None,
            client,
        }
    }

    pub fn with_filters(mut self, filters: ADACFilters) -> Self {
        self.filters = filters;
        self
    }

    pub fn with_direction_classifier(
        mut self,
        classifier: impl Fn(&ADACNewsItem) -> CongestionDirection + 'static,
    ) -> Self {
        self.direction_classifier = Some(Box::new(classifier));
        self
    }
}

impl CongestionSource for ADACSource {
    fn source_id(&self) -> &'static str {
        "adac"
    }

    fn poll(&self) -> Option<scraper::CongestionAmount> {
        let (mut total_inbound_minutes, mut total_outbound_minutes) = (0.0, 0.0);

        let mut has_next_page = true;
        let mut page_number = 1;

        while has_next_page {
            let url = reqwest::Url::parse_with_params(
                "https://www.adac.de/verkehr/verkehrsinformationen/de/baden-wuerttemberg/",
                [
                    ("country", self.country.as_str()),
                    ("federalState", self.federal_state.as_str()),
                    ("street", self.street.street_name()),
                    ("streetType", self.street.street_type()),
                    ("showConstructionSites", "false"),
                    ("pageNumber", page_number.to_string().as_str()),
                    ("submit", "true"),
                    ("resetSearchParams", "false"),
                ],
            )
            .unwrap();

            let result = self
                .client
                .get(url)
                .send()
                .inspect_err(|err| println!("[ADAC source] failed to fetch: {}", err))
                .ok()?
                .text()
                .inspect_err(|err| println!("[ADAC source] failed to parse: {}", err))
                .ok()?;

            let document = html_scraper::Html::parse_document(result.as_str());

            let news_items_selector = html_scraper::Selector::parse(
                "div[data-testid=\"VM-results\"] > div[data-testid=\"VM-news-item\"]",
            )
            .unwrap();

            let news_items_elements = document.select(&news_items_selector);
            if news_items_elements.clone().count() < PAGE_SIZE {
                has_next_page = false;
            }

            let news_items = news_items_elements
                .filter_map(|element| element.child_elements().nth(1))
                .filter_map(|element| {
                    let child_elements = element.child_elements().collect::<Vec<_>>();
                    if child_elements.len() < 3 {
                        return None;
                    }

                    let street_and_sections = &child_elements[0];
                    let street = street_and_sections
                        .child_elements()
                        .nth(0)?
                        .text()
                        .nth(0)?
                        .to_string();
                    let sections = street_and_sections
                        .child_elements()
                        .nth(1)?
                        .text()
                        .map(|s| s.to_string())
                        .collect::<Vec<_>>();

                    let description = child_elements[1].inner_html();
                    let congestion_minutes = child_elements[2]
                        .child_elements()
                        .nth(1)?
                        .text()
                        .find_map(|t| t.ends_with("Minuten").then(|| t.split_once(' ').unwrap().0))?
                        .parse()
                        .ok()?;

                    Some(ADACNewsItem {
                        street,
                        street_sections: sections,
                        description,
                        congestion_minutes,
                    })
                })
                .collect::<Vec<_>>();

            let filtered_items = self.filters.apply(news_items.into_iter());

            let (inbound_minutes, outbound_minutes) = filtered_items
                .map(|item| {
                    let direction = self
                        .direction_classifier
                        .as_ref()
                        .map(|dc| dc(&item))
                        .unwrap_or(CongestionDirection::Both);

                    (item.congestion_minutes, direction)
                })
                .fold(
                    (0.0, 0.0),
                    |(inbound, outbound), (minutes, direction)| match direction {
                        CongestionDirection::Both => (inbound + minutes, outbound + minutes),
                        CongestionDirection::Inbound => (inbound + minutes, outbound),
                        CongestionDirection::Outbound => (inbound, outbound + minutes),
                    },
                );

            total_inbound_minutes += inbound_minutes;
            total_outbound_minutes += outbound_minutes;
            page_number += 1;
        }

        Some(scraper::CongestionAmount::new(
            scraper::CongestionUnit::Minutes(total_inbound_minutes),
            scraper::CongestionUnit::Minutes(total_outbound_minutes),
        ))
    }
}
