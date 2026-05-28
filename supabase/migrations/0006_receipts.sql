create schema if not exists receipts;
create or replace view receipts.candidates as select id, who, did, this, "when", confirmed_by, status from ops.logline_acts where did = 'prepare_receipt_candidate' and status = 'candidate' order by "when" desc;
create or replace view receipts.index as select * from receipts.candidates;
create or replace view receipts.blocked as select id, who, did, this, "when", status from ops.logline_acts where did = 'prepare_receipt_candidate' and not (this ? 'evidence_refs');
create or replace view receipts.closed as select id, who, did, this, "when", status from ops.logline_acts where did in ('close_receipt','emit_receipt') and status in ('closed','emitted');
create or replace view receipts.rejected as select id, who, did, this, "when", status from ops.logline_acts where did = 'reject_receipt_candidate';
