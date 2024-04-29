#!/usr/bin/bash
# rm quebec-latest.osm.pbf
# wget https://download.geofabrik.de/north-america/canada/quebec-latest.osm.pbf -O quebec-latest.osm.pbf

# osm2pgsql -H db -U postgres -d carte -O flex -S import.lua quebec-latest.osm.pbf

psql -h db -U postgres -d carte -c "
                                    drop materialized view if exists bike_path;
                                    CREATE MATERIALIZED VIEW bike_path AS
                                        SELECT *
                                            FROM (
                                                SELECT c.*, cs.score,
                                                ROW_NUMBER() OVER (PARTITION BY c.way_id ORDER BY cs.created_at DESC) as rn
                                                FROM cyclability_score cs 
                                                JOIN cycleway_way c ON c.way_id = ANY(cs.way_ids)
                                            ) t
                                        WHERE t.rn = 1;

                                    CREATE INDEX bike_path_way_id_idx ON bike_path(way_id);
                                    CREATE INDEX edge_geom_gist ON bike_path USING gist(geom);
                                    
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
                                        way_id,
                                        score,
                                        cost_road,
                                        ST_MakeLine(ST_PointN(geom, seq), ST_PointN(geom, seq+1)) as geom,
                                        st_length(ST_MakeLine(ST_PointN(geom, seq), ST_PointN(geom, seq+1))) *
                                        CASE
                                            WHEN score IS NULL THEN 
                                                cost_road
                                            WHEN score = 0 THEN 1 / 0.001
                                            ELSE cost_road * (1 / score)
                                        END as cost,
                                        st_length(ST_MakeLine(ST_PointN(geom, seq), ST_PointN(geom, seq+1))) *
                                        CASE
                                            when tags->>'oneway:bicycle' = 'no' and score is not null and score != 0 then cost_road * (1 / score)
                                            when tags->>'oneway' = 'no' and score is not null and score != 0 then cost_road * (1 / score)
                                            when tags->>'oneway:bicycle' = 'yes' then 1 / 0.001
                                            when tags->>'oneway' = 'yes' then 1 / 0.001
                                            WHEN score IS NULL THEN
                                                cost_road
                                            WHEN score = 0 THEN 1 / 0.001
                                            ELSE cost_road * (1 / score)
                                        END as reverse_cost
                                    from (
                                        select 
                                            way_id, 
                                            unnest(nodes) as node, 
                                            generate_series(1, array_length(nodes, 1)) as seq, 
                                            aw.geom,
                                            score,
                                            aw.name,
                                            aw.tags,
                                            case
                                                when tags->>'bicycle' = 'no' then 1 / 0.0001
                                                when tags->>'highway' = 'cycleway' then 1 / 1
                                                when tags->>'cycleway' = 'track' then 1 / 0.8
                                                when tags->>'cycleway:both' = 'track' then 1 / 0.8
                                                when tags->>'cycleway:left' = 'track' then 1 / 0.8
                                                when tags->>'cycleway:right' = 'track' then 1 / 0.8
                                                when tags->>'cycleway' = 'lane' then 1 / 0.7
                                                when tags->>'cycleway:both' = 'lane' then 1 / 0.7
                                                when tags->>'cycleway:left' = 'lane' then 1 / 0.7
                                                when tags->>'cycleway:right' = 'lane' then 1 / 0.7
                                                when tags->>'cycleway:both' = 'shared_lane' then 1 / 0.6
                                                when tags->>'cycleway:left' = 'shared_lane' then 1 / 0.6
                                                when tags->>'cycleway:right' = 'shared_lane' then 1 / 0.6
                                                when tags->>'cycleway' = 'shared_lane' then 1 / 0.6
                                                when tags->>'highway' = 'residential' then 1 / 0.5
                                                when tags->>'highway' = 'tertiary' then 1 / 0.4
                                                when tags->>'highway' = 'secondary' then 1 / 0.3
                                                when tags->>'highway' = 'service' then 1 / 0.2
                                                when tags->>'bicycle' = 'yes' then 1 / 0.5
                                                when tags->>'bicycle' = 'designated' then 1 / 0.50
                                                when tags->>'highway' = 'primary' then 1 / 0.1
                                                when tags->>'highway' = 'footway' then 1 / 0.1
                                                when tags->>'highway' = 'steps' then 1 / 0.05
                                                when tags->>'highway' = 'proposed' then 1 / 0.001
                                                when tags->>'highway' is not null then 1 / 0.01
                                                else 1 / 0.25
                                            end as cost_road
                                        from all_way aw
                                        left join cyclability_score cs on aw.way_id = any(cs.way_ids)
                                    ) as edges;       

                                    CREATE INDEX edge_way_id_idx ON edge(way_id);
                                    CREATE INDEX edge_source_idx ON edge(source);
                                    CREATE INDEX edge_target_idx ON edge(target);                    
                                    CREATE INDEX edge_x1_idx ON edge(x1);                    
                                    CREATE INDEX edge_y1_idx ON edge(y1);                    
                                    "