#!/usr/bin/env python3
"""Reject incorrect panel pins, inverted row enables and incomplete timing."""
import copy
import importlib.util
import json
from pathlib import Path
import tempfile
import unittest
from unittest.mock import patch

EXT = Path(__file__).resolve().parents[1]
spec = importlib.util.spec_from_file_location("keys_evidence", EXT / "tools/keys-leds-evidence.py")
evidence = importlib.util.module_from_spec(spec)
spec.loader.exec_module(evidence)


class Guards(unittest.TestCase):
    def setUp(self):
        # The fixture records the approved wiring independently of the reader.
        groups = {"clk": [47], "pad_row_n": [15, 16, 27, 28], "pad_col_n": [31, 20, 35, 34],
                  "op_n": [38, 39, 40, 41], "max_din": [17], "max_load": [18], "max_clk": [19],
                  "max_oe": [23], "led_n": [9, 10, 11]}
        lines = []
        for name, pins in groups.items():
            for i, pin in enumerate(pins):
                signal = name if len(pins) == 1 else f"{name}[{i}]"
                lines.append(f'IO_LOC "{signal}" {pin};')
        lines += [f'IO_PORT "pad_row_n[{i}]" IO_TYPE=LVCMOS33 DRIVE=4 TO=SIG;' for i in range(4)]
        lines += ['CLOCK_LOC "machine_clk" BUFG;']
        self.cst = "\n".join(lines)
        self.clocks = {"fmax": {"clk_IBUF_I_O": {"achieved": 148.0, "constraint": 27.0},
                                "machine_clk": {"achieved": 71.0, "constraint": 27.0 / 28}}}
        cells = {f"pad_row_n_TBUF_{i}": {"type": "TBUF", "attributes": {"&TO=SIG": "1", "NEXTPNR_BEL": f"X1Y1/IOB{i}"}} for i in range(4)}
        cells["machine_clk_BUFG"] = {"type": "BUFG", "attributes": {"NEXTPNR_BEL": "X9Y0/BUFG0"}}
        self.module = {"cells": cells}

    def test_reviewed_inputs(self):
        evidence.constraints(self.cst)
        evidence.routed_checks(self.module)
        self.assertEqual(len(evidence.timing(self.clocks)), 2)

    def test_power_pin_rejected(self):
        with self.assertRaisesRegex(ValueError, "pin map"):
            evidence.constraints(self.cst.replace('"pad_row_n[0]" 15', '"pad_row_n[0]" 25'))

    def test_inverted_or_missing_row_polarity(self):
        for replacement in ("TO=INV", ""):
            with self.subTest(replacement=replacement), self.assertRaisesRegex(ValueError, "polarity"):
                evidence.constraints(self.cst.replace("TO=SIG", replacement))

    def test_global_clock_required(self):
        with self.assertRaisesRegex(ValueError, "global machine clock"):
            evidence.constraints(self.cst.replace('CLOCK_LOC "machine_clk" BUFG;', ""))
        altered = copy.deepcopy(self.module)
        altered["cells"].pop("machine_clk_BUFG")
        with self.assertRaisesRegex(ValueError, "global machine clock"):
            evidence.routed_checks(altered)

    def test_routed_polarity_required(self):
        for value in ("0", None):
            altered = copy.deepcopy(self.module)
            attrs = altered["cells"]["pad_row_n_TBUF_0"]["attributes"]
            attrs.pop("&TO=SIG")
            if value is not None:
                attrs["&TO=SIG"] = value
            with self.subTest(value=value), self.assertRaisesRegex(ValueError, "TO=SIG"):
                evidence.routed_checks(altered)

    def test_incomplete_or_failed_timing(self):
        for change in ("missing", "slow", "wrong-frequency", "nan"):
            altered = copy.deepcopy(self.clocks)
            if change == "missing":
                altered["fmax"].pop("machine_clk")
            elif change == "slow":
                altered["fmax"]["machine_clk"]["achieved"] = 0.5
            elif change == "wrong-frequency":
                altered["fmax"]["machine_clk"]["constraint"] = 0.1
            else:
                altered["fmax"]["machine_clk"]["achieved"] = float("nan")
            with self.subTest(change=change), self.assertRaises(ValueError):
                evidence.timing(altered)

    def test_unplaced_cell_rejected(self):
        altered = copy.deepcopy(self.module)
        altered["cells"]["machine_clk_BUFG"]["attributes"].pop("NEXTPNR_BEL")
        with self.assertRaisesRegex(ValueError, "unplaced cell"):
            evidence.routed_checks(altered)

    def test_verify_rejects_stale_pin_and_modified_artifact(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            clone = root / "clone"
            clone.mkdir()
            (root / "HEX1000_COMMIT").write_text("a" * 40)
            (root / "XETAL_COMMIT").write_text("b" * 40)
            (clone / "XETAL_LIBRARIES_COMMIT").write_text("c" * 40)
            strings = root / "work/hex1000-libraries/libs/Strings/src/Strings.xtl"
            strings.parent.mkdir(parents=True)
            strings.write_text("a library")
            image = root / "calculator.fs"
            image.write_text("built image")
            manifest = {"status": "passed", "hardware_actions": [], "project_commit": "a" * 40,
                        "evaluator_commit": "b" * 40, "libraries_commit": "c" * 40,
                        "inputs_sha256": {}, "strings_sha256": evidence.sha(strings),
                        "outputs_sha256": {"calculator.fs": evidence.sha(image)}}
            (root / "evidence.json").write_text(json.dumps(manifest))
            with patch.object(evidence, "ROOT", root), patch.object(evidence, "CLONE", clone):
                evidence.verify(root)
                image.write_text("different image")
                with self.assertRaisesRegex(ValueError, "artifact hash changed"):
                    evidence.verify(root)
                (root / "HEX1000_COMMIT").write_text("d" * 40)
                with self.assertRaisesRegex(ValueError, "source pin changed"):
                    evidence.verify(root)


if __name__ == "__main__":
    unittest.main()
