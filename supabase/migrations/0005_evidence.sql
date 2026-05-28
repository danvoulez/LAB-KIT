create schema if not exists evidence;
create or replace view evidence.records as select id, who, did, this, "when", confirmed_by, status from ops.logline_acts where did in ('report_execution_result','add_evidence','observe','record_evidence') or evidence_state = 'observed' order by "when" desc;
create or replace view evidence.artifacts as select id, this->'artifact_refs' as artifact_refs, "when" from ops.logline_acts where this ? 'artifact_refs';
create or replace view evidence.runtime_observations as select * from evidence.records where did in ('report_execution_result','report_runtime_status');
create or replace view evidence.command_outputs as select id, this->>'stdout_ref' as stdout_ref, this->>'stderr_ref' as stderr_ref, "when" from ops.logline_acts where this ? 'stdout_ref' or this ? 'stderr_ref';
