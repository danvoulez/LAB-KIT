create schema if not exists authz;
create or replace view authz.visas as select * from registry.visas;
