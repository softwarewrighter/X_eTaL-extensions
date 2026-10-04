#!/usr/bin/env bash
# Generate the audio extension's media: test tones and the demo's music,
# all synthesized here (ours, no third-party media). Ogg Vorbis is the
# primary format (plan M6); the tone also as MP3 and WAV.
#   extensions/audio/tests/make-media.sh     (needs sox and lame)
set -euo pipefail
here="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
data="$here/data"
demo="$here/../demos/data"
mkdir -p "$data" "$demo"
tmp="$(mktemp -d)"; trap 'rm -rf "$tmp"' EXIT
# tone: 2 s at 22,050 Hz (the WAV 0.5 s), 440 Hz on the left, 880 Hz on the right
sox -n -r 22050 -c 1 "$tmp/l.wav" synth 2 sine 440 vol 0.5
sox -n -r 22050 -c 1 "$tmp/r.wav" synth 2 sine 880 vol 0.5
sox -M "$tmp/l.wav" "$tmp/r.wav" "$tmp/tone.wav"
sox "$tmp/tone.wav" -C 3 "$data/tone.ogg"
lame --quiet -b 64 "$tmp/tone.wav" "$data/tone.mp3"
sox "$tmp/tone.wav" "$data/tone.wav" trim 0 0.5     # the WAV kept short
# the demo's music: 16 s, a I-vi-IV-V arpeggio over a bass, 44,100 Hz stereo
notes=(C4 E4 G4 C5  A3 C4 E4 A4  F3 A3 C4 F4  G3 B3 D4 G4)
bass=(C2 A1 F1 G1)
parts=()
for bar in 0 1 2 3 4 5 6 7; do
  chord=$((bar % 4))
  for step in 0 1 2 3 4 5 6 7; do
    n="${notes[$((chord * 4 + step % 4))]}"
    f="$tmp/n-$bar-$step.wav"
    if [ "$step" = 0 ] || [ "$step" = 4 ]; then
      sox -n -r 44100 -c 2 "$f" synth 0.25 pluck "$n" pluck "${bass[$chord]}" vol 0.6 fade 0 0.25 0.08
    else
      sox -n -r 44100 -c 2 "$f" synth 0.25 pluck "$n" vol 0.5 fade 0 0.25 0.08
    fi
    parts+=("$f")
  done
done
sox "${parts[@]}" "$tmp/music.wav"
sox "$tmp/music.wav" -C 4 "$demo/arpeggio.ogg" reverb 30
ls -l "$data" "$demo"
