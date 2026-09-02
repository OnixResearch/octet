let Contracts = import "../contracts.ncl" in
((import "../default.ncl") & {
  task_marker_policy.markers | force = [
    { id = "parallel", token = "[parallel]", kind = "literal" },
    { id = "serial", token = "[serial]", kind = "literal" },
    { id = "depends", token = "[depends:<task-or-id>]", kind = "depends-prefix" },
    { id = "task", token = "[task:<id>]", kind = "task-id-prefix" },
  ],
}) | Contracts.Policy
