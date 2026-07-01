-- This view pivots the normalized traffic data into a wide format, producing
-- one row per timestamp. Each source gets its own inbound and outbound columns,
-- making the data easier to query and visualize in dashboards or time series charts.

CREATE VIEW traffic_visualization AS
SELECT
    timestamp,
    MAX(CASE WHEN source_id='radio7' THEN congestion_amount_inbound_minutes END)  AS radio7_inbound,
    MAX(CASE WHEN source_id='radio7' THEN congestion_amount_outbound_minutes END) AS radio7_outbound,
    MAX(CASE WHEN source_id='adac' THEN congestion_amount_inbound_minutes END)    AS adac_inbound,
    MAX(CASE WHEN source_id='adac' THEN congestion_amount_outbound_minutes END)   AS adac_outbound
FROM congestion_entries
GROUP BY timestamp;
