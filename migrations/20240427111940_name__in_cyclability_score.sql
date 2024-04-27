ALTER TABLE cyclability_score
ADD COLUMN name TEXT[];

UPDATE cyclability_score cs
SET name = (
  SELECT ARRAY_AGG(cw.name)
  FROM cycleway_way cw
  WHERE cw.way_id = ANY(cs.way_ids)
);