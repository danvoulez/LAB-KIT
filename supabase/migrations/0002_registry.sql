create schema if not exists registry;
create table if not exists registry.entities (
  entity_id text primary key,
  entity_type text not null default 'unknown',
  source_act_id uuid references ops.logline_acts(id),
  profile jsonb not null default '{}'::jsonb,
  created_at timestamptz not null default now()
);
create table if not exists registry.runtimes (
  runtime_id text primary key,
  runtime_type text not null default 'unknown',
  source_act_id uuid references ops.logline_acts(id),
  profile jsonb not null default '{}'::jsonb,
  created_at timestamptz not null default now()
);
create table if not exists registry.links (link_id text primary key, source_act_id uuid references ops.logline_acts(id), payload jsonb not null default '{}'::jsonb);
create table if not exists registry.passports (passport_id text primary key, entity_id text, source_act_id uuid references ops.logline_acts(id), payload jsonb not null default '{}'::jsonb);
create table if not exists registry.visas (visa_id text primary key, entity_id text, source_act_id uuid references ops.logline_acts(id), payload jsonb not null default '{}'::jsonb);
create table if not exists registry.auth_bindings (binding_id text primary key, entity_id text, source_act_id uuid references ops.logline_acts(id), payload jsonb not null default '{}'::jsonb);
