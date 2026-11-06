#!/usr/bin/env python3
"""Compiler evidence must justify dependency, fusion and launch diagrams."""
from collections import Counter
import hashlib
import importlib.util
import io
import json
from pathlib import Path
import sys
import unittest

EXT = Path(__file__).resolve().parents[1]
DATA = EXT / "demos/gpu-build-data"
spec = importlib.util.spec_from_file_location("gpu_build", EXT / "tools/gpu-build.py")
adapter = importlib.util.module_from_spec(spec)
spec.loader.exec_module(adapter)


class Evidence(unittest.TestCase):
    def setUp(self):
        self.text = (DATA / "checked.txt").read_text()
        self.nodes, self.edges = adapter.checked(self.text)
        self.plan = (DATA / "m1-pipeline.plan").read_text()

    def test_real_dependency_graph_and_fusion(self):
        p = adapter.plan(self.plan, self.nodes)
        self.assertEqual(len(self.nodes), 16)
        self.assertEqual(len(self.edges), 16)
        self.assertEqual(Counter(n["kind"] for n in self.nodes), {0: 7, 1: 2, 2: 2, 3: 2, 4: 1, 5: 2})
        self.assertEqual([len(k["values"]) for k in p["kernels"]], [3, 1, 4, 1])
        self.assertEqual(len(p["buffers"]), 8)
        self.assertEqual([l["global"] for l in p["launches"]], [8, 256, 8, 256])

    def test_missing_dependency_rejected(self):
        with self.assertRaisesRegex(ValueError, "dependency"):
            adapter.checked(self.text.replace("map mul %x %w", "map mul %absent %w"))

    def test_duplicate_value_rejected(self):
        with self.assertRaisesRegex(ValueError, "duplicate"):
            adapter.checked(self.text.replace("%w :", "%x :"))

    def test_missing_kernel_coverage_rejected(self):
        with self.assertRaisesRegex(ValueError, "coverage"):
            adapter.plan(self.plan.replace("computes %mul %add %max", "computes %mul %max"), self.nodes)

    def test_materialized_fusion_intermediate_rejected(self):
        bad = self.plan.replace("buffers:\n", "buffers:\n  b_mul: long 8 items, %mul i64 [8]; written on the device\n")
        with self.assertRaisesRegex(ValueError, "intermediate"):
            adapter.plan(bad, self.nodes)

    def test_nondivisible_launch_rejected(self):
        with self.assertRaisesRegex(ValueError, "invalid launch"):
            adapter.plan(self.plan.replace("global 8 local 8", "global 8 local 3"), self.nodes)

    def test_zero_launch_rejected(self):
        with self.assertRaisesRegex(ValueError, "invalid launch"):
            adapter.plan(self.plan.replace("global 8 local 8", "global 8 local 0"), self.nodes)

    def test_published_pin_and_bytes(self):
        manifest = json.loads((DATA / "manifest.json").read_text())
        self.assertEqual(manifest["gpu_commit"], (adapter.ROOT / "GPU_COMMIT").read_text().strip())
        self.assertFalse(manifest["device_execution"])
        for name, digest in manifest["artifacts_sha256"].items():
            self.assertEqual(hashlib.sha256((DATA / name).read_bytes()).hexdigest(), digest, name)


if __name__ == "__main__":
    log = io.StringIO()
    result = unittest.TextTestRunner(stream=log).run(unittest.defaultTestLoader.loadTestsFromTestCase(Evidence))
    if not result.wasSuccessful():
        sys.stderr.write(log.getvalue())
        sys.exit(1)
    print(f"PASS {result.testsRun} GPU compiler-evidence checks")
