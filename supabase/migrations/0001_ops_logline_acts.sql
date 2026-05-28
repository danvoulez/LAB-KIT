create extension if not exists pgcrypto;
create schema if not exists ops;

create table if not exists ops.logline_acts (
  id uuid primary key default gen_random_uuid(),
  who text not null,
  did text not null,
  this jsonb not null,
  "when" timestamptz not null default now(),
  confirmed_by jsonb not null default '{}'::jsonb,
  if_ok jsonb not null default '{}'::jsonb,
  if_doubt jsonb not null default '{}'::jsonb,
  if_not jsonb not null default '{}'::jsonb,
  status text not null,
  runtime_envelope jsonb not null default '{}'::jsonb,
  tuple_hash text,
  content_hash text,
  previous_act_refs text[] not null default '{}',
  evidence_state text not null default 'declared',
  promotion_state text not null default 'candidate',
  created_at timestamptz not null default now(),
  constraint logline_act_status_nonempty check (length(trim(status)) > 0),
  constraint logline_act_who_nonempty check (length(trim(who)) > 0),
  constraint logline_act_did_nonempty check (length(trim(did)) > 0)
);

create index if not exists logline_acts_when_idx on ops.logline_acts ("when" desc);
create index if not exists logline_acts_did_idx on ops.logline_acts (did);
create index if not exists logline_acts_status_idx on ops.logline_acts (status);
create index if not exists logline_acts_this_gin_idx on ops.logline_acts using gin (this);
