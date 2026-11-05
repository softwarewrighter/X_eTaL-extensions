#!/usr/bin/env python3
"""Reject incomplete physical-calculator scene evidence."""
from collections import Counter
import importlib.util
import io
from pathlib import Path
import sys
import unittest

spec = importlib.util.spec_from_file_location("build_arrays", Path(__file__).resolve().parents[1] / "tools/keys-leds-build.py")
adapter = importlib.util.module_from_spec(spec)
spec.loader.exec_module(adapter)


class BuildArrays(unittest.TestCase):
    def setUp(self):
        types = ["LUT4", "DFFR", "ALU", "SP", "DP", "IBUF", "OBUF", "MUX2_LUT5", "TBUF"]
        bels = ["X1Y1/LUT0", "X1Y1/DFF0", "X2Y1/ALU1", "X1Y2/BSRAM", "X2Y2/BSRAM", "X0Y1/IOBA", "X0Y2/IOBB", "X1Y1/MUX5", "X0Y3/IOBA"]
        self.cells = [dict(name=f"cell{i}", type=t, bel=b) for i, (t, b) in enumerate(zip(types, bels))]
        self.routes = {"net": "X1Y1/F0;;1;X2Y1/A0;X2Y1/A0/F0;1"}
        self.build = {"placed_counts": dict(Counter(types)), "routed_nets": 1, "bitstream_bytes": 64,
                      "utilization": {k: {"used": n} for k, n in zip(("LUT4", "DFF", "ALU", "BSRAM", "IOB"), (1, 1, 1, 2, 2))},
                      "fmax_mhz": {"clk": {"achieved": 30, "constraint": 27}, "machine_clk": {"achieved": 2, "constraint": 27 / 28}}}

        self.build["utilization"]["TBUF"] = {"used": 1}

    def check(self):
        return adapter.arrays(self.cells, self.routes, self.build)

    def test_all_resources_and_exact_tiles(self):
        rows, tiles, stats = self.check()
        self.assertEqual([r[3] for r in rows], [0, 1, 2, 3, 3, 4, 4, 5, 4])
        self.assertEqual(tiles, [(1, 1), (2, 1)])
        self.assertEqual(stats[:9], [1, 1, 9, 2, 1, 64, 1, 2, 3])

    def test_missing_bel(self):
        self.cells[0]["bel"] = ""
        with self.assertRaisesRegex(ValueError, "BEL"):
            self.check()

    def test_duplicate_identity(self):
        self.cells[1]["name"] = self.cells[0]["name"]
        with self.assertRaisesRegex(ValueError, "identity"):
            self.check()

    def test_wrong_resource_count(self):
        self.build["utilization"]["BSRAM"]["used"] = 1
        with self.assertRaisesRegex(ValueError, "resource totals"):
            self.check()

    def test_missing_route_coordinates(self):
        self.routes["net"] = "no coordinates"
        with self.assertRaisesRegex(ValueError, "coordinates"):
            self.check()

    def test_machine_timing_failure(self):
        self.build["fmax_mhz"]["machine_clk"]["achieved"] = 0.5
        with self.assertRaisesRegex(ValueError, "timing failed"):
            self.check()

    def test_tri_state_rows_count_separately(self):
        self.build["utilization"]["TBUF"]["used"] = 0
        with self.assertRaisesRegex(ValueError, "tri-state"):
            self.check()


if __name__ == "__main__":
    log = io.StringIO()
    result = unittest.TextTestRunner(stream=log).run(unittest.defaultTestLoader.loadTestsFromTestCase(BuildArrays))
    if not result.wasSuccessful():
        sys.stderr.write(log.getvalue())
        sys.exit(1)
    print(f"PASS {result.testsRun} physical build-array checks")
