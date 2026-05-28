create schema if not exists lab_observability;
create or replace view lab_observability.ghosts as select * from audit.v_open_ghosts;
create or replace view lab_observability.current_state as select * from audit.v_daily_lab_state;
create or replace view lab_observability.heartbeats as select id, who, this, "when" from ops.logline_acts where did in ('heartbeat','machine_heartbeat') order by "when" desc;
create or replace view lab_observability.runtime_status as select id, who, this, "when", status from ops.logline_acts where did in ('register_runtime','report_runtime_status') order by "when" desc;
create or replace view lab_observability.machine_state as select * from lab_observability.runtime_status;
