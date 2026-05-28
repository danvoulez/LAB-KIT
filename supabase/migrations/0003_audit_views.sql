create schema if not exists audit;
create or replace view audit.v_recent_acts as select * from ops.logline_acts order by "when" desc limit 100;
create or replace view audit.v_open_ghosts as select id, who, did, this, "when", status from ops.logline_acts where did = 'open_ghost' and status in ('open','declared') order by "when" desc;
create or replace view audit.v_receipt_candidates as select id, who, did, this, "when", status from ops.logline_acts where did = 'prepare_receipt_candidate' order by "when" desc;
create or replace view audit.v_daily_lab_state as
select now() as observed_at,
       (select count(*) from ops.logline_acts) as acts,
       (select count(*) from audit.v_open_ghosts) as open_ghosts,
       (select count(*) from audit.v_receipt_candidates) as receipt_candidates;
create or replace view audit.v_mobile_today as select * from audit.v_daily_lab_state;
