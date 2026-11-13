#!/usr/bin/env python3
"""Capture fresh signed Cairn conflict receipts from minimal, isolated Git roots."""
import argparse
import base64
import json
import os
from pathlib import Path
import shutil
import struct
import subprocess
import tempfile

CHANGE = "exercise-cairn-cob-conflict-recovery"
HERE = Path(__file__).resolve().parent




def run(args, cwd=None, expected=0):
    result = subprocess.run([str(arg) for arg in args], cwd=cwd, capture_output=True)
    if result.returncode != expected:
        raise RuntimeError(f"{args}: exit {result.returncode}; expected {expected}\n"
                           + result.stdout.decode() + result.stderr.decode())
    return result.stdout


def git(root, *args):
    return run(["git", "-C", root, *args]).decode().strip()


def actor(key):
    raw = base64.b64decode(key.with_suffix(".pub").read_text().split()[1])
    size = struct.unpack(">I", raw[:4])[0]
    public = raw[4 + size + 4:]
    assert len(public) == 32
    number = int.from_bytes(b"\xed\x01" + public, "big")
    alphabet = "123456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz"
    encoded = ""
    while number:
        number, remainder = divmod(number, 58)
        encoded = alphabet[remainder] + encoded
    return "z" + encoded


def digest(path):
    return run(["b3sum", path]).decode().split()[0]


def nickel(value, depth=0):
    if isinstance(value, dict):
        indent = "  " * depth
        return "{\n" + "".join("  " + indent + key + " = " + nickel(item, depth + 1)
                                 + ",\n" for key, item in value.items()) + indent + "}"
    if isinstance(value, list):
        return "[" + ", ".join(nickel(item, depth) for item in value) + "]"
    return json.dumps(value, ensure_ascii=False)


def capture(cairn, source_revision, scratch, output):
    evidence = output / "evidence"
    evidence.mkdir(parents=True, exist_ok=True)
    (output / "heads").mkdir(exist_ok=True)
    commands = []

    def init(root, bare=False):
        root.mkdir()
        git(root, "init", "--initial-branch=main", *(["--bare"] if bare else []))
        git(root, "config", "user.name", "Octet conflict fixture")
        git(root, "config", "user.email", "fixture@example.invalid")

    def cli(name, namespace, command, root, store, key=None, extra=(), execute=False,
            expected_verdict="PASS"):
        args = [cairn, namespace, command, CHANGE, "--git-repository", store,
                "--root", root, "--policy", root / "cairn-policy/generated/cairn-policy.json"]
        if namespace != "cob":
            args += ["--backend", "radicle-cob.v1"]
        if key:
            args += ["--keystore", key]
        args += list(extra)
        if execute:
            args += ["--execute"]
        commands.append([str(arg).replace(str(scratch), "<scratch>") for arg in args])
        data = run(args)
        receipt = json.loads(data)
        (evidence / name).write_bytes(data)
        assert receipt["verdict"] == expected_verdict, receipt
        return receipt

    creator_key = scratch / "creator-keys"
    delegate_key = scratch / "delegate-keys"
    for directory in (creator_key, delegate_key):
        directory.mkdir(mode=0o700)
        run(["ssh-keygen", "-q", "-t", "ed25519", "-N", "", "-C", "ephemeral-conflict-fixture",
             "-f", directory / "radicle"])
    creator_actor = actor(creator_key / "radicle")
    delegate_actor = actor(delegate_key / "radicle")

    source = scratch / "source-root"
    init(source)
    shutil.copytree(HERE / "inputs", source, dirs_exist_ok=True)
    policy = source / "cairn-policy/generated/cairn-policy.json"
    policy.parent.mkdir(parents=True, exist_ok=True)
    policy.write_bytes((HERE.parents[1] / "cairn-policy/generated/cairn-policy.json").read_bytes())
    git(source, "add", ".")
    env = os.environ.copy()
    env.update(GIT_AUTHOR_DATE="2026-11-12T00:00:00Z", GIT_COMMITTER_DATE="2026-11-12T00:00:00Z")
    committed = subprocess.run(["git", "-C", str(source), "commit", "-m",
                                "Clean minimal Octet conflict lifecycle fixture"],
                               env=env, capture_output=True)
    if committed.returncode:
        raise RuntimeError(committed.stderr.decode())
    base = git(source, "rev-parse", "HEAD")
    creator_store = scratch / "creator.git"
    delegate_store = scratch / "delegate.git"
    convergence = scratch / "convergence.git"
    reload_store = scratch / "reload.git"
    for store in (creator_store, delegate_store, convergence, reload_store):
        init(store, bare=True)
        git(store, "fetch", "--no-tags", source, "refs/heads/main:refs/heads/main")
    delegated = scratch / "delegate-root"
    third = scratch / "third-root"
    for root in (delegated, third):
        init(root)
        git(root, "symbolic-ref", "HEAD", "refs/heads/fixture-unborn")
        git(root, "fetch", "--no-tags", source, "refs/heads/main:refs/heads/main")
        git(root, "checkout", "main")

    import_extra = ["--workspace", "workspace:octet", "--delegate", delegate_actor]
    imported_preview = cli("import-preview.json", "cob", "import", source, creator_store,
                           creator_key, import_extra)
    imported = cli("import-execute.json", "cob", "import", source, creator_store,
                   creator_key, import_extra, execute=True)
    assert imported["plan_hash"] == imported_preview["plan_hash"]
    obj = imported["object_id"]
    creator_ref = f"refs/namespaces/{creator_actor}/refs/cobs/xyz.onixresearch.cairn-change/{obj}"
    delegate_ref = f"refs/namespaces/{delegate_actor}/refs/cobs/xyz.onixresearch.cairn-change/{obj}"
    git(delegate_store, "fetch", "--no-tags", creator_store, f"{creator_ref}:{creator_ref}")

    updates = []
    for label, root, store, key in [("creator", source, creator_store, creator_key),
                                    ("delegate", delegated, delegate_store, delegate_key)]:
        tasks = HERE / "heads" / f"{label}-tasks.md"
        shutil.copyfile(tasks, root / ".cairn/changes" / CHANGE / "tasks.md")
        if tasks != output / "heads" / tasks.name:
            shutil.copyfile(tasks, output / "heads" / tasks.name)
        preview = cli(f"{label}-update-preview.json", "collaboration", "update", root, store, key)
        updated = cli(f"{label}-update-execute.json", "collaboration", "update", root, store,
                      key, execute=True)
        assert preview["plan_hash"] == updated["plan_hash"]
        assert updated["input_state_hash"] == imported["state_hash"]
        updates.append(updated)
    creator_update, delegate_update = updates
    creator_oid = creator_update["result_operation_id"]
    delegate_oid = delegate_update["result_operation_id"]
    transport_commands = []

    def transport(destination, origin, *refspecs):
        args = ["git", "-C", str(destination), "fetch", "--no-tags", str(origin), *refspecs]
        transport_commands.append([arg.replace(str(scratch), "<scratch>") for arg in args])
        run(args)

    transport(convergence, creator_store, f"+{creator_ref}:{creator_ref}")
    transport(convergence, delegate_store, f"+{delegate_ref}:{delegate_ref}")
    metadata = cli("conflicts-metadata.json", "collaboration", "conflicts", source, convergence)
    assert metadata["conflicted"] is True
    observed_heads = [head["revision_id"] for head in metadata["artifact_conflicts"][0]["heads"]]
    assert sorted(observed_heads) == sorted([creator_oid, delegate_oid]), observed_heads
    for label, oid in [("creator", creator_oid), ("delegate", delegate_oid)]:
        shown = cli(f"conflicts-show-{label}.json", "collaboration", "conflicts", source,
                    convergence, extra=["--show-head", f"tasks={oid}"])
        assert shown["shown_head"]["content"].encode() == (output / "heads" / f"{label}-tasks.md").read_bytes()
    refused = cli("update-conflict-negative.json", "collaboration", "update", source, convergence,
                  creator_key, expected_verdict="FAIL")
    assert refused["blocked"] and refused["dry_run"] and not refused["mutated"]
    assert any("conflict" in blocker for blocker in refused["blockers"]), refused
    resolve_extra = ["--choose-head", f"tasks={creator_oid}"]
    resolve_preview = cli("resolve-preview.json", "collaboration", "resolve", source,
                          convergence, creator_key, resolve_extra)
    resolved = cli("resolve-execute.json", "collaboration", "resolve", source, convergence,
                   creator_key, resolve_extra, execute=True)
    assert resolve_preview["plan_hash"] == resolved["plan_hash"]
    resolved_oid = resolved["result_operation_id"]
    parents = git(convergence, "show", "-s", "--format=%P", resolved_oid).split()
    assert sorted(parents) == sorted([creator_oid, delegate_oid]), parents
    git(convergence, "merge-base", "--is-ancestor", delegate_oid, resolved_oid)

    bundle = output / "octet-cob-conflict.bundle"
    git(convergence, "bundle", "create", bundle, "refs/heads/main", creator_ref, delegate_ref)
    transport(reload_store, bundle, "+refs/heads/main:refs/heads/main",
              "+refs/namespaces/*:refs/namespaces/*")
    shutil.rmtree(third / ".cairn/changes" / CHANGE)
    before = cli("status-third-before-checkout.json", "collaboration", "status", third, reload_store)
    assert before["local_destination"] == "absent"
    conflicts = cli("conflicts-resolved-third.json", "collaboration", "conflicts", third, reload_store)
    assert conflicts["conflicted"] is False
    cli("checkout-third-preview.json", "collaboration", "checkout", third, reload_store)
    checkout = cli("checkout-third-execute.json", "collaboration", "checkout", third,
                   reload_store, execute=True)
    assert checkout["post_checkout_destination"] == "current"
    after = cli("status-third-after-checkout.json", "collaboration", "status", third, reload_store)
    assert after["lifecycle"]["condition"] == "stale", after["lifecycle"]
    assert (third / ".cairn/changes" / CHANGE / "tasks.md").read_bytes() == (output / "heads/creator-tasks.md").read_bytes()
    remotes = [git(store, "remote").splitlines() for store in (creator_store, convergence, reload_store)]
    assert remotes == [[], [], []]
    transport_receipt = {
        "schema": "octet.cairn-cob-conflict-transport.v1", "commands": transport_commands,
        "backend": "radicle-cob.v1", "change": CHANGE, "cairn_commit": source_revision,
        "octet_base_git_oid": base, "object_id": obj,
        "creator": {"actor": creator_actor, "ref_name": creator_ref,
                    "conflict_target": creator_oid, "resolved_target": git(reload_store, "rev-parse", creator_ref)},
        "delegate": {"actor": delegate_actor, "ref_name": delegate_ref,
                     "conflict_target": delegate_oid, "resolved_target": git(reload_store, "rev-parse", delegate_ref)},
        "resolution": {"operation_id": resolved_oid, "direct_parents": parents,
                       "selected_revision": creator_oid, "losing_revision_reachable": True},
        "source_remotes": remotes[0], "convergence_remotes": remotes[1], "reload_remotes": remotes[2],
        "valid": True, "verdict": "PASS",
        "non_claims": [
            "matching local Git refs and ancestry prove bounded local transport observations only",
            "local path and bundle transport do not prove authentication, remote authorization, publication, availability, durability, synchronization, distributed convergence, or remote trust",
            "explicit head selection and complete ancestry do not prove semantic merge correctness or that the losing revision is invalid",
            "matching lifecycle bytes do not prove gate validity, evidence truth, source correctness, whole-program correctness, or release eligibility"],
    }
    (evidence / "transport.json").write_text(json.dumps(transport_receipt, indent=2) + "\n")

    def head(label, updated, actor_id, ref):
        tasks = output / "heads" / f"{label}-tasks.md"
        return {"actor": actor_id, "ref_name": ref, "operation_id": updated["result_operation_id"],
                "content_blake3": digest(tasks), "content_bytes": tasks.stat().st_size,
                "content_root": updated["result_content_root"], "state_hash": updated["result_state_hash"],
                "retained_path": f"heads/{label}-tasks.md"}

    heads = [{"target": target, "ref_name": ref}
             for target, ref in (line.split() for line in git(convergence, "bundle", "list-heads", bundle).splitlines())]
    manifest = {
        "schema": "octet.cairn-cob-conflict-pilot.v1", "change": CHANGE, "workspace_id": "workspace:octet",
        "backend": "radicle-cob.v1", "cairn_commit": source_revision, "octet_base_git_oid": base,
        "policy_hash": imported["policy_hash"], "object_id": obj,
        "root_operation_id": imported["operation_id"], "root_content_root": imported["content_root"],
        "root_state_hash": imported["state_hash"],
        "creator": head("creator", creator_update, creator_actor, creator_ref),
        "delegate": head("delegate", delegate_update, delegate_actor, delegate_ref),
        "conflict_state_hash": metadata["state_hash"],
        "resolution": {"operation_id": resolved_oid, "selected_revision": creator_oid,
                       "losing_revision": delegate_oid, "direct_parents": parents,
                       "content_root": resolved["result_content_root"], "state_hash": resolved["result_state_hash"]},
        "resolved_lifecycle_condition": after["lifecycle"]["condition"],
        "bundle": {"path": bundle.name, "blake3_hex": digest(bundle), "heads": heads},
        "receipts": [{"name": path.name, "blake3_hex": digest(path)} for path in sorted(evidence.glob("*.json"))],
        "required_non_claim_fragments": ["semantic merge correctness", "synchronization", "distributed convergence", "durability", "release eligibility"],
    }
    source_manifest = (HERE / "manifest.ncl").read_text()
    types = source_manifest.split("let CreatorTasksBytes")[0].split("# Recorded observation")[0]
    (output / "manifest.ncl").write_text(types + "# Recorded observation: fresh pinned-provider execution.\n"
                                         + nickel(manifest) + " | ConflictPilotManifest\n")
    (output / "manifest.json").write_bytes(run(["nickel", "export", "--format", "json", output / "manifest.ncl"]))
    (output / "replay-commands.json").write_text(json.dumps(commands, indent=2) + "\n")
    provenance = {
        "source_revision": source_revision,
        "executable": str(Path(cairn).resolve()),
        "executable_blake3": digest(Path(cairn)),
        "observation_kind": "fresh signed CLI capture; not a historical receipt migration",
        "base_has_parents": bool(git(source, "show", "-s", "--format=%P", base)),
        "base_paths": git(source, "ls-tree", "-r", "--name-only", base).splitlines(),
    }
    assert provenance["base_has_parents"] is False
    (output / "provider-provenance.json").write_text(json.dumps(provenance, indent=2) + "\n")
    print(json.dumps({"base_ref": "refs/heads/main", "base_oid": base, "object_id": obj,
                      "creator_ref": creator_ref, "creator_update": creator_oid,
                      "delegate_ref": delegate_ref, "delegate_update": delegate_oid,
                      "resolution": resolved_oid, "output": str(output)}, indent=2))


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--cairn", required=True)
    parser.add_argument("--cairn-source-revision", required=True)
    parser.add_argument("--output", type=Path, required=True)
    options = parser.parse_args()
    output = options.output.resolve()
    output.mkdir(parents=True, exist_ok=True)
    os.umask(0o077)
    with tempfile.TemporaryDirectory(prefix="octet-cob-conflict-") as directory:
        scratch = Path(directory)
        captured = scratch / "captured"
        captured.mkdir()
        capture(options.cairn, options.cairn_source_revision, scratch, captured)
        shutil.copytree(captured, output, dirs_exist_ok=True)


if __name__ == "__main__":
    main()
