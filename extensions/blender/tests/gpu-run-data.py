#!/usr/bin/env python3
"""Reject broken execution evidence before a scene consumes it."""
import copy
import importlib.util
import io
import json
from pathlib import Path
import sys
import unittest

EXT = Path(__file__).resolve().parents[1]
spec = importlib.util.spec_from_file_location("gpu_run", EXT / "tools/gpu-run.py")
adapter = importlib.util.module_from_spec(spec)
spec.loader.exec_module(adapter)
DATA = EXT / "demos/gpu-run-data"


class Trace(unittest.TestCase):
    def setUp(self):
        self.trace = json.loads((DATA / "rtx.json").read_text())
        self.record = copy.deepcopy(self.trace["records"][2])

    def test_all_actual_trials(self):
        for record in self.trace["records"]:
            adapter.observer.validate(record)

    def test_bad_output(self):
        self.record["outputs"][0] = 33
        with self.assertRaisesRegex(ValueError, "output"):
            adapter.observer.validate(self.record)

    def test_missing_launch(self):
        self.record["events"].pop(4)
        with self.assertRaisesRegex(ValueError, "trace"):
            adapter.observer.validate(self.record)

    def test_wrong_transfer_bytes(self):
        self.record["events"][0]["bytes"] = 32
        with self.assertRaisesRegex(ValueError, "trace"):
            adapter.observer.validate(self.record)

    def test_reversed_timestamp(self):
        self.record["events"][0]["end_ns"] = 0
        with self.assertRaisesRegex(ValueError, "timestamps"):
            adapter.observer.validate(self.record)

    def test_queue_overlap(self):
        self.record["events"][1].update(self.record["events"][0])
        self.record["events"][1]["name"] = "b_w"
        with self.assertRaisesRegex(ValueError, "timestamps"):
            adapter.observer.validate(self.record)

    def test_nonfinite_host_phase(self):
        self.record["host_ms"]["bind"] = float("nan")
        with self.assertRaisesRegex(ValueError, "host"):
            adapter.observer.validate(self.record)

    def test_host_phase_accounting(self):
        self.record["host_ms"]["total"] = 0
        with self.assertRaisesRegex(ValueError, "host"):
            adapter.observer.validate(self.record)

    def test_published_hashes_and_reduction(self):
        import contextlib
        with contextlib.redirect_stdout(io.StringIO()):
            adapter.verify(DATA)
        values = list(map(float, (DATA / "reduction.txt").read_text().split()))
        self.assertEqual(len(values), 9*256)
        self.assertEqual(values[-256], 32)
        self.assertEqual(values[:8], [2,0,6,0,10,0,14,0])


if __name__ == "__main__":
    log = io.StringIO()
    result = unittest.TextTestRunner(stream=log).run(unittest.defaultTestLoader.loadTestsFromTestCase(Trace))
    if not result.wasSuccessful():
        sys.stderr.write(log.getvalue())
        sys.exit(1)
    print(f"PASS {result.testsRun} GPU execution-evidence checks")
