create table cyclability_score (
    way_id int8 not null,
    score integer not null,
    created_at timestamp not null default now(),
    primary key (way_id, created_at)
);

CREATE INDEX idx_cyclability_score_created_at ON cyclability_score(created_at);