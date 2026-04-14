#!/usr/bin/env python3
from pathlib import Path
import tomllib

repo_root = Path(__file__).resolve().parents[4]
path = repo_root / "templates" / "dylint.toml"
data = tomllib.loads(path.read_text())
assert data == {"tigerstyle": {}}, data
print(f"parsed {path}")
print("expected table shape: {'tigerstyle': {}}")
