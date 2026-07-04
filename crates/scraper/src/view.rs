use std::collections::HashMap;

use chrono::{Datelike, Timelike};
use itertools::Itertools;

use crate::{CongestionAmount, CongestionSource};

pub struct CongestionTrainingInput {
    pub time_sin: f64,
    pub time_cos: f64,

    pub weekday_sin: f64,
    pub weekday_cos: f64,

    pub month_sin: f64,
    pub month_cos: f64,

    pub is_weekend: bool,
}

impl From<chrono::DateTime<chrono::Utc>> for CongestionTrainingInput {
    fn from(timestamp: chrono::DateTime<chrono::Utc>) -> Self {
        let minute_of_day = timestamp.hour() as f64 * 60.0 + timestamp.minute() as f64;
        let weekday = timestamp.weekday() as u8;
        let month = timestamp.month() as u8;

        Self {
            month_sin: (month as f64 / 12.0 * 2.0 * std::f64::consts::PI).sin(),
            month_cos: (month as f64 / 12.0 * 2.0 * std::f64::consts::PI).cos(),

            weekday_sin: (weekday as f64 / 7.0 * 2.0 * std::f64::consts::PI).sin(),
            weekday_cos: (weekday as f64 / 7.0 * 2.0 * std::f64::consts::PI).cos(),

            time_sin: (minute_of_day / 1440.0 * 2.0 * std::f64::consts::PI).sin(),
            time_cos: (minute_of_day / 1440.0 * 2.0 * std::f64::consts::PI).cos(),

            is_weekend: timestamp.weekday() == chrono::Weekday::Sat
                || timestamp.weekday() == chrono::Weekday::Sun,
        }
    }
}

impl CongestionTrainingInput {
    pub const N_FEATURES: usize = 7;

    pub fn into_features(self) -> [f64; Self::N_FEATURES] {
        [
            self.time_sin,
            self.time_cos,
            self.weekday_sin,
            self.weekday_cos,
            self.month_sin,
            self.month_cos,
            if self.is_weekend { 1.0 } else { 0.0 },
        ]
    }
}

pub struct CongestionTrainingPoint {
    pub input: CongestionTrainingInput,
    pub congestion: f64,
}

impl CongestionTrainingPoint {
    fn new(input: impl Into<CongestionTrainingInput>, congestion: f64) -> Self {
        Self {
            input: input.into(),
            congestion,
        }
    }
}

pub struct CongestionView {
    timestamps: Vec<chrono::DateTime<chrono::Utc>>,
    source_entries: HashMap<&'static str, Vec<Option<CongestionAmount>>>,
}

impl CongestionView {
    pub fn timestamped_values_for(
        &self,
        source_id: &str,
    ) -> impl Iterator<Item = (chrono::DateTime<chrono::Utc>, Option<CongestionAmount>)> {
        let source_entries = self
            .source_entries
            .get(source_id)
            .expect("Source not found");

        self.timestamps
            .iter()
            .zip(source_entries.iter())
            .map(|(timestamp, entry)| (*timestamp, *entry))
    }

    pub fn timestamps(&self) -> impl Iterator<Item = chrono::DateTime<chrono::Utc>> {
        self.timestamps.iter().copied()
    }

    pub fn num_points(&self) -> usize {
        self.timestamps.len()
    }

    pub fn training_points(
        &self,
        source_id: &str,
        inbound: bool,
    ) -> impl Iterator<Item = CongestionTrainingPoint> {
        self.timestamped_values_for(source_id)
            .map(move |(timestamp, entry)| {
                CongestionTrainingPoint::new(
                    timestamp,
                    entry.map_or(0.0, |e| {
                        (if inbound { e.inbound } else { e.outbound }).as_minutes()
                    }),
                )
            })
    }
}

impl CongestionView {
    pub(crate) fn make_view(
        sources: &[Box<dyn CongestionSource>],
        conn: &rusqlite::Connection,
    ) -> rusqlite::Result<Self> {
        conn.execute("DROP VIEW IF EXISTS congestion_view", ())?;
        conn.execute(
            format!(
                "CREATE VIEW congestion_view AS
                SELECT
                    timestamp{}
                FROM congestion_entries
                GROUP BY timestamp",
                sources
                    .iter()
                    .map(|src| format!(
                        ",
                        MAX(CASE WHEN source_id='{0}' THEN congestion_amount_inbound_minutes END) AS {0}_inbound,
                        MAX(CASE WHEN source_id='{0}' THEN congestion_amount_outbound_minutes END) AS {0}_outbound",
                        src.source_id()
                    )).join("")
            )
            .as_str(),
            (),
        )?;

        let mut count_stmt = conn.prepare("SELECT COUNT(*) FROM congestion_view")?;
        let count = count_stmt
            .query_map([], |row| row.get::<_, i64>(0))
            .unwrap()
            .next()
            .unwrap()
            .unwrap() as usize;

        let mut timestamps = Vec::with_capacity(count);
        let mut source_entries: HashMap<&'static str, Vec<Option<CongestionAmount>>> = sources
            .iter()
            .map(|src| (src.source_id(), Vec::with_capacity(count)))
            .collect();

        let mut data_stmt = conn.prepare("SELECT * FROM congestion_view")?;
        let mut data_iterator = data_stmt.query([])?;

        while let Ok(Some(row)) = data_iterator.next() {
            let timestamp =
                chrono::DateTime::parse_from_rfc3339(row.get::<_, String>(0).unwrap().as_str())
                    .unwrap()
                    .to_utc();

            timestamps.push(timestamp);

            for source in sources {
                let inbound = row
                    .get::<_, Option<f64>>(format!("{}_inbound", source.source_id()).as_str())
                    .expect("Inbound value missing");
                let outbound = row
                    .get::<_, Option<f64>>(format!("{}_outbound", source.source_id()).as_str())
                    .expect("Outbound value missing");

                let entry = inbound
                    .zip(outbound)
                    .map(|(inbound, outbound)| CongestionAmount::new_minutes(inbound, outbound));

                source_entries
                    .get_mut(source.source_id())
                    .unwrap()
                    .push(entry);
            }
        }

        Ok(Self {
            timestamps,
            source_entries,
        })
    }
}
