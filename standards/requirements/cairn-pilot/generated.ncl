# Generated source record for the Cairn pilot contract.
#
# Every value here is tool output written by commands into generated-input.json
# (plain `b3sum` over files in the Octet checkout at revision
# d5b34c2d64b1227417a46df8ed8427f2d10ed3c8); nothing is hand-typed. See README.md
# beside this file for the command and the matching producer proof-card entries.
let Cairn = import "../../../cairn-policy/assurance-targets.ncl" in

let input = import "./generated-input.json" in

{
  schema_version = "cairn.assurance-target-sources.v1",
  specifications = input.specifications,
  entries = input.entries,
} | Cairn.GeneratedSources
