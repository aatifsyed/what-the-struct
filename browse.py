#!/usr/bin/env python3
from pathlib import Path
import json
from ptpython.entry_points.run_ptipython import run
from typing import Any

krate = json.loads(Path("examples/foo/target/doc/foo.json").read_text())
std = json.loads(Path("vendor/std.json").read_text())


def paths(d: dict[str, Any]) -> dict[str, Any]:
    d2 = dict()
    for k, v in d["paths"].items():
        kind = v["kind"]
        if kind in ("enum", "struct", "union"):
            d2[k] = v
    return d2


def index(d: dict[str, Any]) -> dict[str, Any]:
    d2 = dict()
    for k, v in d["index"].items():
        kind = v["kind"]
        if kind in ("enum", "struct", "union", "variant", "struct_field"):
            d2[k] = v
    return d2


run(user_ns=locals())
