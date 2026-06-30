use std::{collections::HashMap, sync::LazyLock};

use scraper::{CongestionDirection, CongestionSource, CongestionUnit};

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[allow(unused)]
pub struct Radio7ApiTraffic {
    pub title: String,
    pub description: String,
    pub road_type: String,
    pub road_letter: String,
    pub road_number: Option<u32>,
    pub road_name: String,
    pub keywords: Vec<String>,
}

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct Radio7ApiResponse {
    traffic: HashMap<String, Vec<Radio7ApiTraffic>>,
}

#[derive(Default)]
pub struct Radio7Filters {
    title_contains: Vec<String>,
    road_name_equals: Vec<String>,
}

impl Radio7Filters {
    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title_contains.push(title.into().to_lowercase());
        self
    }

    pub fn road_name(mut self, road_name: impl Into<String>) -> Self {
        self.road_name_equals.push(road_name.into());
        self
    }
}

impl Radio7Filters {
    fn apply(
        &self,
        items: impl Iterator<Item = Radio7ApiTraffic>,
    ) -> impl Iterator<Item = Radio7ApiTraffic> {
        items.filter(|item| {
            if !self
                .title_contains
                .iter()
                .any(|title| item.title.to_lowercase().contains(title))
            {
                return false;
            }

            if !self
                .road_name_equals
                .iter()
                .any(|road_name| &item.road_name == road_name)
            {
                return false;
            }

            true
        })
    }
}

static REGEX: LazyLock<regex::Regex> = LazyLock::new(|| {
    regex::RegexBuilder::new(r"(?<minutes>\d) minuten")
        .case_insensitive(true)
        .build()
        .unwrap()
});

#[derive(Default)]
pub struct Radio7Source {
    filters: Radio7Filters,
    direction_classifier: Option<Box<dyn Fn(&Radio7ApiTraffic) -> CongestionDirection>>,
}

impl Radio7Source {
    pub fn new() -> Self {
        Self {
            filters: Radio7Filters::default(),
            direction_classifier: None,
        }
    }

    pub fn with_filters(mut self, filters: Radio7Filters) -> Self {
        self.filters = filters;
        self
    }

    pub fn with_direction_classifier(
        mut self,
        classifier: impl Fn(&Radio7ApiTraffic) -> CongestionDirection + 'static,
    ) -> Self {
        self.direction_classifier = Some(Box::new(classifier));
        self
    }
}

impl CongestionSource for Radio7Source {
    fn source_id(&self) -> &'static str {
        "radio7"
    }

    fn poll(&self) -> Option<scraper::CongestionAmount> {
        let all_warnings: Radio7ApiResponse =
            reqwest::blocking::get("https://www.radio7.de/traffic")
                .inspect_err(|err| {
                    println!("[RADIO7 source] Failed to fetch traffic: {}", err);
                })
                .and_then(|res| res.json())
                .inspect_err(|err| {
                    println!("[RADIO7 source] Failed to parse traffic: {}", err);
                })
                .ok()?;

        let (inbound_minutes, outbound_minutes): (f64, f64) = self
            .filters
            .apply(all_warnings.traffic.into_values().flat_map(|values| values))
            .filter_map(|item| {
                let congestion_minutes = REGEX.captures(&item.title)?;
                let minutes_match = congestion_minutes.name("minutes")?;
                let minutes = minutes_match.as_str().parse::<f64>().ok()?;

                let direction = self
                    .direction_classifier
                    .as_ref()
                    .map(|dc| dc(&item))
                    .unwrap_or(CongestionDirection::Both);

                Some((minutes, direction))
            })
            .fold(
                (0.0, 0.0),
                |(inbound, outbound), (minutes, direction)| match direction {
                    CongestionDirection::Both => (inbound + minutes, outbound + minutes),
                    CongestionDirection::Inbound => (inbound + minutes, outbound),
                    CongestionDirection::Outbound => (inbound, outbound + minutes),
                },
            );

        Some(scraper::CongestionAmount::new(
            CongestionUnit::Minutes(inbound_minutes),
            CongestionUnit::Minutes(outbound_minutes),
        ))
    }
}
