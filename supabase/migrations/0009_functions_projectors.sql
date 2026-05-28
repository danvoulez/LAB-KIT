create schema if not exists audit;
-- v0 projectors are SQL views declared by migrations 0003-0007. This file is intentionally idempotent.
create or replace function audit.projector_health()
returns jsonb language sql stable as $$
  select jsonb_build_object('projectors','sql-views','spine','ops.logline_acts','checked_at', now())
$$;
