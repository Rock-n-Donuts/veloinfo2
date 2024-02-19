CREATE TABLE if not exists photo (
	"timestamp" timestamp NOT NULL,
	"path" varchar NOT NULL,
    cyclability_score_id integer NOT NULL,
    FOREIGN KEY (cyclability_score_id) REFERENCES cyclability_score(id)
);
CREATE INDEX if not exists photo_timestamp_idx ON public.photo ("timestamp");