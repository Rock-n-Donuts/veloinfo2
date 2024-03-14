#!/usr/bin/bash
rm quebec-latest.osm.pbf
wget https://download.geofabrik.de/north-america/canada/quebec-latest.osm.pbf -O quebec-latest.osm.pbf

osm2pgsql -H db -U postgres -d carte -O flex -S import.lua quebec-latest.osm.pbf

psql -h db -U postgres -d carte -c "CREATE OR REPLACE VIEW bike_path AS
                                        SELECT *
                                            FROM (
                                                SELECT c.*, cs.score,
                                                ROW_NUMBER() OVER (PARTITION BY c.way_id ORDER BY cs.created_at DESC) as rn
                                                FROM cyclability_score cs 
                                                JOIN cycleway_way c ON c.way_id = ANY(cs.way_ids)
                                            ) t
                                        WHERE t.rn = 1;"