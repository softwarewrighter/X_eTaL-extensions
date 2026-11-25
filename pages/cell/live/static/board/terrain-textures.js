// The textures terrain.js makes when a game has no file of that name:
// grass, snow, dirt, stone, sand, mud. Each is SIZE by SIZE pixels,
// worked out once from noise that repeats across the edge (so it tiles)
// and from a fixed seed (so it is the same picture every time). The
// red, green and blue of a pixel are its color; its fourth number is
// how high the surface is there (128 the middle), which terrain.js uses
// to make the edge between two textures ragged: the second lies first
// in the hollows of the first.
export const SIZE = 256;

// numbers from 0 to 1, the same ones every time for a seed (mulberry32)
const random = (seed) => () => {
  seed = (seed + 0x6d2b79f5) | 0;
  let t = Math.imul(seed ^ (seed >>> 15), 1 | seed);
  t = (t + Math.imul(t ^ (t >>> 7), 61 | t)) ^ t;
  return ((t ^ (t >>> 14)) >>> 0) / 4294967296;
};

// Noise over the whole texture, 0 to 1: `cells` lumps across, and
// `octaves` finer layers each half as strong, every one repeating
// across the edge.
function noise(cells, octaves, seed) {
  const out = new Float32Array(SIZE * SIZE), next = random(seed);
  let strength = 1, total = 0;
  for (let o = 0; o < octaves && cells <= SIZE; o++, cells *= 2, strength *= 0.5) {
    const grid = new Float32Array(cells * cells);
    for (let i = 0; i < grid.length; i++) grid[i] = next();
    const step = cells / SIZE;
    for (let y = 0; y < SIZE; y++) {
      const gy = y * step, y0 = Math.floor(gy), y1 = (y0 + 1) % cells, fy = gy - y0, sy = fy * fy * (3 - 2 * fy);
      for (let x = 0; x < SIZE; x++) {
        const gx = x * step, x0 = Math.floor(gx), x1 = (x0 + 1) % cells, fx = gx - x0, sx = fx * fx * (3 - 2 * fx);
        const a = grid[y0 * cells + x0], b = grid[y0 * cells + x1], c = grid[y1 * cells + x0], d = grid[y1 * cells + x1];
        out[y * SIZE + x] += strength * (a + (b - a) * sx + (c - a) * sy + (a - b - c + d) * sx * sy);
      }
    }
    total += strength;
  }
  for (let i = 0; i < out.length; i++) out[i] /= total;
  return out;
}

const hex = (c) => [parseInt(c.slice(1, 3), 16), parseInt(c.slice(3, 5), 16), parseInt(c.slice(5, 7), 16)];
const clamp = (v) => (v < 0 ? 0 : v > 1 ? 1 : v);

// A texture from a rule a pixel: rule(i, x, y) gives how far from the
// dark color to the light one (0 to 1), how bright (about 1), and how
// high (0 to 1).
function paint(dark, light, rule) {
  const d = hex(dark), l = hex(light), data = new Uint8Array(SIZE * SIZE * 4);
  for (let y = 0, i = 0; y < SIZE; y++) {
    for (let x = 0; x < SIZE; x++, i++) {
      const [mix, bright, high] = rule(i, x, y), m = clamp(mix);
      for (let k = 0; k < 3; k++) data[i * 4 + k] = Math.round(255 * clamp((d[k] + (l[k] - d[k]) * m) * bright / 255));
      data[i * 4 + 3] = Math.round(255 * clamp(high));
    }
  }
  return data;
}

const RECIPES = {
  // tufts in patches, and blades: fine speckle
  grass() {
    const patch = noise(4, 3, 11), tuft = noise(24, 2, 12), blade = noise(96, 2, 13);
    return paint('#46592f', '#7d8f4c', (i) => [0.55 * patch[i] + 0.45 * tuft[i], 0.8 + 0.4 * blade[i], 0.35 * tuft[i] + 0.65 * blade[i]]);
  },
  // soft drifts, a fine grain
  snow() {
    const drift = noise(4, 3, 21), grain = noise(128, 1, 22);
    return paint('#c9d5e2', '#f4f7fa', (i) => [0.25 + 0.75 * drift[i], 0.97 + 0.06 * grain[i], 0.7 * drift[i] + 0.3 * grain[i]]);
  },
  // earth with small stones
  dirt() {
    const patch = noise(5, 4, 31), grain = noise(128, 1, 32), stone = noise(40, 1, 33);
    return paint('#4c4033', '#7f6c56', (i) => {
      const pebble = stone[i] > 0.82 ? 1 : 0;
      return [0.8 * patch[i] + 0.2 * grain[i] + 0.35 * pebble, 0.85 + 0.3 * grain[i], 0.5 * patch[i] + 0.2 * grain[i] + 0.3 * pebble];
    });
  },
  // gray rock, cracked in places where a second noise crosses its middle
  stone() {
    const face = noise(5, 4, 41), vein = noise(4, 2, 42), some = noise(3, 2, 44), grain = noise(128, 1, 43);
    return paint('#636668', '#a3a4a1', (i) => {
      const crack = clamp(1 - Math.abs(2 * vein[i] - 1) * 30) * clamp((some[i] - 0.4) * 5);
      return [face[i] * (1 - 0.6 * crack), (0.9 + 0.2 * grain[i]) * (1 - 0.3 * crack), face[i] * (1 - crack)];
    });
  },
  // grains, and ripples the wind left
  sand() {
    const dune = noise(3, 3, 51), grain = noise(128, 1, 52);
    return paint('#ad9666', '#dac896', (i, x, y) => {
      const ripple = 0.5 + 0.5 * Math.sin(2 * Math.PI * (10 * y / SIZE + x / SIZE + 1.5 * dune[i]));
      return [0.6 * dune[i] + 0.15 * ripple + 0.25 * grain[i], 0.92 + 0.16 * grain[i], 0.2 * ripple + 0.5 * dune[i] + 0.3 * grain[i]];
    });
  },
  // dark wet earth, darker in its hollows
  mud() {
    const lump = noise(6, 4, 61), grain = noise(64, 2, 62);
    return paint('#33281e', '#66533e', (i) => {
      const pool = clamp((0.42 - lump[i]) * 6);
      return [lump[i] * (1 - 0.6 * pool), (0.9 + 0.2 * grain[i]) * (1 - 0.25 * pool), lump[i]];
    });
  },
};

// the names this module can make
export const KNOWN = Object.keys(RECIPES);

// The pixels of a known name (four numbers a pixel, a Uint8Array), or
// null for a name it does not know.
export function make(name) {
  return RECIPES[name] ? RECIPES[name]() : null;
}
