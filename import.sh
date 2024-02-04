#!/usr/bin/bash
rm quebec-latest.osm.pbf
wget https://download.geofabrik.de/north-america/canada/quebec-latest.osm.pbf -O quebec-latest.osm.pbf

osm2pgsql -H db -U postgres -d carte -O flex -S import.lua quebec-latest.osm.pbf

psql -h db -U postgres -d carte -c "create or replace view bike_path as
    select cycleway.*, recent_cyclability_score.score 
    from cycleway 
    left join (
        SELECT *
        FROM cyclability_score
        WHERE (way_ids, created_at) IN (
            SELECT way_ids, MAX(created_at)
            FROM cyclability_score
            GROUP BY way_ids
        )
        order by created_at desc
    )AS recent_cyclability_score ON cycleway.way_id = any(recent_cyclability_score.way_ids);"