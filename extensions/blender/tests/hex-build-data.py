#!/usr/bin/env python3
"""Check placement evidence, route triples, and missing-artifact failures."""
import importlib.util
import json
from pathlib import Path
import tempfile
import unittest

ROOT = Path(__file__).resolve().parents[3]
spec = importlib.util.spec_from_file_location("hexbuild", ROOT / "scripts/hex1000-build.py")
build = importlib.util.module_from_spec(spec)
spec.loader.exec_module(build)


class Evidence(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.directory = Path(self.temp.name)
        self.cells = {
            "sum": {"type": "LUT4", "attributes": {"NEXTPNR_BEL": "X3Y7/LUT2"}},
            "state": {"type": "DFFRE", "attributes": {"NEXTPNR_BEL": "X3Y7/DFF2"}},
            "carry": {"type": "ALU", "attributes": {"NEXTPNR_BEL": "X3Y7/ALU0"}},
        }
        self.routing = "X3Y7/Q2;;1;X4Y7/W100;X4Y7/W100/Q2;1"
        for name in ("design.v", "design.cst", "hex1000.fs"):
            (self.directory / name).write_text("test artifact\n")

    def export(self):
        (self.directory / "synth.json").write_text(json.dumps({"modules": {"hex1000_top": {"cells": self.cells}}}))
        (self.directory / "pnr.json").write_text(json.dumps({"modules": {"top": {
            "cells": self.cells,
            "netnames": {"sum": {"attributes": {"ROUTING": self.routing}}},
        }}}))
        build.export(self.directory)

    def test_identity_and_shared_tile(self):
        self.export()
        self.assertEqual((self.directory / "cells.txt").read_text(), "3 7 2 1\n3 7 2 0\n")
        self.assertEqual((self.directory / "routes.txt").read_text(), "3 7\n4 7\n")
        manifest = json.loads((self.directory / "evidence.json").read_text())
        self.assertEqual([c["name"] for c in manifest["cells"]], ["state", "sum"])
        self.assertEqual(manifest["synthesis_counts"]["ALU"], 1)
        self.assertEqual(len(manifest["sha256"]["hex1000.fs"]), 64)

    def test_missing_placement_rejected(self):
        self.cells["state"]["attributes"] = {}
        with self.assertRaisesRegex(ValueError, "unplaced logic cell: state"):
            self.export()

    def test_malformed_routes_rejected(self):
        self.routing = "X3Y7/Q2;missing-strength"
        with self.assertRaisesRegex(ValueError, "ROUTING format"):
            self.export()

    def test_unrouted_design_rejected(self):
        self.routing = " "
        with self.assertRaisesRegex(ValueError, "no routed wire"):
            self.export()


if __name__ == "__main__":
    unittest.main()
