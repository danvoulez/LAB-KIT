create schema if not exists workorders;
create or replace view workorders.dispatch_packets as select id, who, did, this, "when", status from ops.logline_acts where did = 'prepare_dispatch_packet';
create or replace view workorders.hermes_workorders as select id, who, did, this, "when", status from ops.logline_acts where did = 'prepare_hermes_workorder';
create or replace view workorders.execution_reports as select id, who, did, this, "when", status from ops.logline_acts where did = 'report_execution_result';
