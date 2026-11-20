# Cairn pilot contract: Octet's Aeneas external-consumer fixture over the
# realized Mantle producer output.
#
# The projection is Cairn's: `project` turns the authored assurance targets plus
# the generated source record into Octet's `octet-requirement-contract/v1`.
# Export deterministic JSON with:
#   nickel export --format json standards/requirements/cairn-pilot/contract.ncl \
#     | jq -S . > standards/requirements/cairn-pilot/contract.json
let Cairn = import "../../../cairn-policy/assurance-targets.ncl" in

let targets = import "./targets.ncl" in
let generated = import "./generated.ncl" in

Cairn.project (std.array.first targets.contracts) generated
