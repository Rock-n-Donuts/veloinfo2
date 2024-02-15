drop view if exists bike_path;

ALTER TABLE cyclability_score
ALTER COLUMN created_at TYPE timestamptz 
USING created_at AT TIME ZONE 'UTC';

CREATE OR REPLACE VIEW bike_path AS
SELECT *
    FROM (
        SELECT cycleway.*, recent_cyclability_score.score,
        ROW_NUMBER() OVER (PARTITION BY cycleway.way_id ORDER BY recent_cyclability_score.created_at DESC) as rn
        FROM cycleway 
        LEFT JOIN cyclability_score AS recent_cyclability_score ON cycleway.way_id = ANY(recent_cyclability_score.way_ids)
    ) t
WHERE t.rn = 1;