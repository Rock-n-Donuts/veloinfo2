-- Add migration script here
CREATE TABLE if not exists cyclability_score_history (
	score float8 NOT NULL,
	created_at timestamp DEFAULT now() NOT NULL,
	way_ids _int8 NULL,
	"comment" text NULL,
	id serial4 NOT NULL,
	CONSTRAINT cyclability_score_history_pkey PRIMARY KEY (id)
);
CREATE index if not exists idx_cyclability_score_history_created_at ON cyclability_score_history USING btree (created_at);
CREATE INDEX if not exists idx_way_ids ON cyclability_score_history USING gin (way_ids);