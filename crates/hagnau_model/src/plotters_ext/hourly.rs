use chrono::Timelike;
use plotters::coord::ranged1d::{NoDefaultFormatting, Ranged, ValueFormatter};
use std::{ops::Range, ops::Sub};

pub struct ChronoHourly {
    range: Range<chrono::DateTime<chrono::Utc>>,
}

impl ValueFormatter<chrono::DateTime<chrono::Utc>> for ChronoHourly {
    fn format(value: &chrono::DateTime<chrono::Utc>) -> String {
        format!("{:02}:{:02}", value.hour(), value.minute())
    }
}

impl Ranged for ChronoHourly {
    type FormatOption = NoDefaultFormatting;

    type ValueType = chrono::DateTime<chrono::Utc>;

    fn map(&self, value: &Self::ValueType, limit: (i32, i32)) -> i32 {
        let total_span = self.range.end.sub(self.range.start);
        let value_span = value.sub(self.range.start);

        // First, lets try the nanoseconds precision
        if let Some(total_ns) = total_span.num_nanoseconds() {
            if let Some(value_ns) = value_span.num_nanoseconds() {
                return (f64::from(limit.1 - limit.0) * value_ns as f64 / total_ns as f64) as i32
                    + limit.0;
            }
        }

        // Yes, converting them to floating point may lose precision, but this is Ok.
        // If it overflows, it means we have a time span nearly 300 years, we are safe to ignore the
        // portion less than 1 day.
        let total_days = total_span.num_days() as f64;
        let value_days = value_span.num_days() as f64;

        (f64::from(limit.1 - limit.0) * value_days / total_days) as i32 + limit.0
    }

    fn key_points<Hint: plotters::coord::ranged1d::KeyPointHint>(
        &self,
        _hint: Hint,
    ) -> Vec<Self::ValueType> {
        let mut current_date = self.range.start.with_hour(0).unwrap();
        if current_date < self.range.start {
            current_date += chrono::Duration::days(1);
        }

        let end_date = self.range.end;

        let mut points = vec![];

        while current_date <= end_date {
            points.push(current_date);
            current_date += chrono::Duration::hours(1);
        }

        points
    }

    fn range(&self) -> std::ops::Range<Self::ValueType> {
        self.range.start..self.range.end
    }
}

pub trait IntoChronoHourly {
    fn hourly(self) -> ChronoHourly;
}

impl IntoChronoHourly for Range<chrono::DateTime<chrono::Utc>> {
    fn hourly(self) -> ChronoHourly {
        ChronoHourly { range: self }
    }
}
