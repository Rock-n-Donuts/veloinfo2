ALTER TABLE cyclability_score
ADD COLUMN geom GEOMETRY;

UPDATE cyclability_score cs
SET geom = (
  SELECT ST_Union(cw.geom)
  FROM cycleway_way cw
  WHERE cw.way_id = ANY(cs.way_ids)
);