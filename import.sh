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
                                        WHERE t.rn = 1;
                                        
                                        drop materialized view if exists edge;
                                        drop sequence if exists edge_id;
                                        CREATE SEQUENCE edge_id;
                                        CREATE MATERIALIZED VIEW edge 
                                        AS SELECT  
                                            nextval('edge_id')  as id, 
                                            node as source,
                                            lead(node) over (partition by way_id order by seq) as target,
                                            st_x(st_transform(ST_PointN(geom, seq), 4326)) as x1,
                                            st_y(st_transform(ST_PointN(geom, seq), 4326)) as y1,
                                            st_x(st_transform(ST_PointN(geom, seq+1), 4326)) as x2,
                                            st_y(st_transform(ST_PointN(geom, seq+1), 4326)) as y2,
                                            way_id
                                        from (
                                            select way_id, unnest(nodes) as node, generate_series(1, array_length(nodes, 1)) as seq, geom
                                            from cycleway_way
                                        ) as edges;                           
                                        "