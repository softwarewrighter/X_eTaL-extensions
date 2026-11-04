#!/usr/bin/env python3
"""MAX7219 framing must reject short words and invalid commands/data."""
import importlib.util
from pathlib import Path
import tempfile
import unittest

EXT = Path(__file__).resolve().parents[1]
spec = importlib.util.spec_from_file_location("keys_wave", EXT / "tools/keys-leds-run.py")
wave = importlib.util.module_from_spec(spec)
spec.loader.exec_module(wave)


def fixture(word, count=16, high_load=False, unknown=False):
    text = "$timescale\n1ps\n$end\n$var wire 1 a din $end\n$var wire 1 b sclk $end\n$var wire 1 c load $end\n$enddefinitions $end\n#0\n0a\n0b\n1c\n#1\n"
    text += "1c\n" if high_load else "0c\n"
    tick = 2
    for i in range(count):
        bit = "x" if unknown and i == 0 else str((word >> (15 - i)) & 1)
        text += f"#{tick}\n{bit}a\n#{tick+1}\n1b\n#{tick+2}\n0b\n"
        tick += 3
    return text + f"#{tick}\n1c\n"


class Decoder(unittest.TestCase):
    def decode(self, text):
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "test.vcd"
            path.write_text(text)
            return wave.read_wave(path, list(range(1, 17)), [0x0c01])

    def test_digit_latches_on_load(self):
        waves, words, screens = self.decode(fixture(0x0101))
        self.assertEqual(words[0]["word"], 0x0101)
        self.assertEqual(screens[-1][1].strip(), "0")
        self.assertEqual(waves["display0"][-1][1], 1)
        self.assertEqual(waves["spi_bits"][-1][1], 16)

    def test_partial_word_rejected(self):
        with self.assertRaisesRegex(ValueError, "16 bits"):
            self.decode(fixture(0x0101, count=15))

    def test_clock_requires_load_low(self):
        with self.assertRaisesRegex(ValueError, "clock/data"):
            self.decode(fixture(0x0101, high_load=True))

    def test_unknown_data_rejected(self):
        with self.assertRaisesRegex(ValueError, "clock/data"):
            self.decode(fixture(0x0101, unknown=True))

    def test_invalid_setup_rejected(self):
        with self.assertRaisesRegex(ValueError, "initialization"):
            self.decode(fixture(0x9901))


if __name__ == "__main__":
    unittest.main()
