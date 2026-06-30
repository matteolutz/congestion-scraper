use crate::CongestionAmount;

#[derive(Debug)]
pub struct CongestionEntry {
    timestamp: chrono::DateTime<chrono::Utc>,
    source_id: String,

    congestion_amount: CongestionAmount,
}

impl CongestionEntry {
    pub fn new(
        timestamp: chrono::DateTime<chrono::Utc>,
        source_id: &'static str,
        congestion_amount: CongestionAmount,
    ) -> Self {
        Self {
            timestamp,
            source_id: source_id.to_string(),
            congestion_amount,
        }
    }
}

impl CongestionEntry {
    pub(crate) fn ensure_table(conn: &rusqlite::Connection) -> rusqlite::Result<()> {
        conn.execute(
            "CREATE TABLE IF NOT EXISTS congestion_entries (
                timestamp                           DATETIME NOT NULL,
                source_id                           TEXT NOT NULL,
                congestion_amount_inbound_minutes   REAL NOT NULL,
                congestion_amount_outbound_minutes  REAL NOT NULL,

                PRIMARY KEY (timestamp, source_id)
            )",
            (),
        )?;

        Ok(())
    }

    pub(crate) fn find_all(conn: &rusqlite::Connection) -> rusqlite::Result<Vec<CongestionEntry>> {
        let mut stmt = conn.prepare("SELECT * FROM congestion_entries")?;

        let entries = stmt
            .query_map([], |row| {
                Ok(CongestionEntry {
                    timestamp: chrono::DateTime::parse_from_rfc3339(
                        row.get::<_, String>(0)?.as_str(),
                    )
                    .unwrap()
                    .to_utc(),
                    source_id: row.get(1)?,
                    congestion_amount: CongestionAmount {
                        inbound: crate::CongestionUnit::Minutes(row.get(2)?),
                        outbound: crate::CongestionUnit::Minutes(row.get(3)?),
                    },
                })
            })?
            .filter_map(|result| result.ok());

        Ok(entries.collect::<Vec<_>>())
    }

    pub(crate) fn count(conn: &rusqlite::Connection) -> rusqlite::Result<usize> {
        let mut stmt = conn.prepare("SELECT COUNT(*) FROM congestion_entries")?;
        let count = stmt.query_row([], |row| row.get::<_, i32>(0))?;
        Ok(count as usize)
    }

    pub(crate) fn insert(self, conn: &rusqlite::Connection) -> rusqlite::Result<()> {
        conn.execute(
            "INSERT INTO congestion_entries (timestamp, source_id, congestion_amount_inbound_minutes, congestion_amount_outbound_minutes)
            VALUES (?, ?, ?, ?)",
            (
                self.timestamp.to_rfc3339(),
                self.source_id,
                self.congestion_amount.inbound.as_minutes(),
                self.congestion_amount.outbound.as_minutes(),
            ),
        )?;

        Ok(())
    }
}
