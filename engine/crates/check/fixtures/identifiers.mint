14 schemes

decision_id
  pattern    {namespace}-DR-{seq:04d}
  namespace  HW
  render     HW-DR-0000
  needs      seq:4
  mint       HW-DR-0042
  admits     true
  sequence   42
  refuses    true

acceptance_criterion_id
  pattern    {namespace}-AC-{seq:04d}
  namespace  HW
  render     HW-AC-0000
  needs      seq:4
  mint       HW-AC-0042
  admits     true
  sequence   42
  refuses    true

evaluation_id
  pattern    {namespace}-EVAL-{slug}
  namespace  HW
  render     HW-EVAL-<slug>
  needs      slug
  mint       HW-EVAL-a-name
  admits     true
  sequence   -
  refuses    false

interface_contract_id
  pattern    {namespace}-IFACE-{slug}
  namespace  HW
  render     HW-IFACE-<slug>
  needs      slug
  mint       HW-IFACE-a-name
  admits     true
  sequence   -
  refuses    false

obligation_record_id
  pattern    {namespace}-OBL-{seq:04d}
  namespace  HW
  render     HW-OBL-0000
  needs      seq:4
  mint       HW-OBL-0042
  admits     true
  sequence   42
  refuses    true

probe_id
  pattern    {namespace}-PROBE-{slug}
  namespace  HW
  render     HW-PROBE-<slug>
  needs      slug
  mint       HW-PROBE-a-name
  admits     true
  sequence   -
  refuses    false

process_decision_id
  pattern    {namespace}-PD-{seq:04d}
  namespace  HW
  render     HW-PD-0000
  needs      seq:4
  mint       HW-PD-0042
  admits     true
  sequence   42
  refuses    true

register_id
  pattern    {namespace}-REG-{slug}
  namespace  HW
  render     HW-REG-<slug>
  needs      slug
  mint       HW-REG-a-name
  admits     true
  sequence   -
  refuses    false

requirement_id
  pattern    {namespace}-REQ-{seq:04d}
  namespace  HW
  render     HW-REQ-0000
  needs      seq:4
  mint       HW-REQ-0042
  admits     true
  sequence   42
  refuses    true

result_id
  pattern    {namespace}-RESULT-{slug}
  namespace  HW
  render     HW-RESULT-<slug>
  needs      slug
  mint       HW-RESULT-a-name
  admits     true
  sequence   -
  refuses    false

review_id
  pattern    {namespace}-REV-{slug}
  namespace  HW
  render     HW-REV-<slug>
  needs      slug
  mint       HW-REV-a-name
  admits     true
  sequence   -
  refuses    false

run_id
  pattern    {namespace}-RUN-{slug}
  namespace  HW
  render     HW-RUN-<slug>
  needs      slug
  mint       HW-RUN-a-name
  admits     true
  sequence   -
  refuses    false

spec_id
  pattern    {namespace}-SPEC-{slug}
  namespace  HW
  render     HW-SPEC-<slug>
  needs      slug
  mint       HW-SPEC-a-name
  admits     true
  sequence   -
  refuses    false

tutorial_id
  pattern    {namespace}-TUT-{slug}
  namespace  HW
  render     HW-TUT-<slug>
  needs      slug
  mint       HW-TUT-a-name
  admits     true
  sequence   -
  refuses    false
