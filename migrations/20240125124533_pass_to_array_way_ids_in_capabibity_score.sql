alter table
    cyclability_score
add column if not exists
     way_ids int8 [];

CREATE INDEX if not exists idx_way_ids ON cyclability_score USING gin(way_ids);

UPDATE cyclability_score SET way_ids = ARRAY[way_id];

ALTER TABLE cyclability_score drop if exists way_id CASCADE ;
