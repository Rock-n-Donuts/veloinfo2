ALTER TABLE public.cyclability_score ADD "user_id" uuid NULL;

CREATE INDEX idx_cyclability_score_user ON public.cyclability_score("user_id");