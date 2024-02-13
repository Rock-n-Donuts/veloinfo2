#!/usr/bin/bash
rm quebec-latest.osm.pbf
wget https://download.geofabrik.de/north-america/canada/quebec-latest.osm.pbf -O quebec-latest.osm.pbf

osm2pgsql -H db -U postgres -d carte -O flex -S import.lua quebec-latest.osm.pbf

psql -h db -U postgres -d carte -c "CREATE OR REPLACE VIEW bike_path AS
                                        SELECT *
                                            FROM (
                                                SELECT cycleway.*, recent_cyclability_score.score,
                                                ROW_NUMBER() OVER (PARTITION BY cycleway.way_id ORDER BY recent_cyclability_score.created_at DESC) as rn
                                                FROM cycleway 
                                                LEFT JOIN cyclability_score AS recent_cyclability_score ON cycleway.way_id = ANY(recent_cyclability_score.way_ids)
                                            ) t
                                        WHERE t.rn = 1;"