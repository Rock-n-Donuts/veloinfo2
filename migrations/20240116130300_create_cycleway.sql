CREATE TABLE if not exists public.cycleway (
	way_id int8 NOT NULL,
	"name" text NULL,
	winter_service text NULL,
	geom public.geometry(linestring, 3857) NULL,
	"source" int8 NULL,
	target int8 NULL,
	kind text NULL
);
CREATE INDEX if not exists cycleway_geom_idx ON public.cycleway using gist (geom);