alter table ops.logline_acts enable row level security;

do $$ begin
  if not exists (select 1 from pg_policies where schemaname='ops' and tablename='logline_acts' and policyname='logline_acts_no_public_direct_access') then
    create policy logline_acts_no_public_direct_access on ops.logline_acts for all using (false) with check (false);
  end if;
end $$;
