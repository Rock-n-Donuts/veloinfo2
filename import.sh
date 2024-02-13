#!/usr/bin/bash
rm quebec-latest.osm.pbf
wget https://download.geofabrik.de/north-america/canada/quebec-latest.osm.pbf -O quebec-latest.osm.pbf

osm2pgsql -H db -U postgres -d carte -O flex -S import.lua quebec-latest.osm.pbf

psql -h db -U postgres -d carte -c "CREATE OR REPLACE VIEW bike_path AS
                                        select cycleway.*, cyclability_score.score 
                                        from cycleway 
                                        left join cyclability_score ON cycleway.way_id = any(cyclability_score.way_ids);"