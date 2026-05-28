alter table ops.logline_acts enable row level security;
do $$ begin
  if not exists (select 1 from pg_policies where schemaname='ops' and tablename='logline_acts' and policyname='logline_acts_service_all') then
    create policy logline_acts_service_all on ops.logline_acts for all using (true) with check (true);
  end if;
end $$;
