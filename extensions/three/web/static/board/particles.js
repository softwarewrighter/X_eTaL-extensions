// The board's particles, a module of the board player (../board.js says
// what a module is). One kept call:
//   particles(kind, nums('x z y', 3)[0], rate, nums('wx wz', 2)[0], seed)
// kind: 'smoke', 'dust', 'splash', 'fire' or 'embers'; the place they
// come from (x east, z south, y up, in tiles); rate, particles a second;
// the wind, tiles a second along x and z; a seed.
//
// Nothing is stepped frame by frame. Particle number n of an emitter is
// born at (n + j) / rate seconds of the board's clock (j in 0..1, a hash
// of n and the seed), and where it is, how large and what color are
// functions of its age, n, the seed and the emitter's numbers, worked
// out in the vertex shader from the clock (one uniform). So a still at
// ?t=5 is the same picture every time, and a line that is unchanged is
// untouched however often its scene is shown.
//
// When it starts: an emitter's particles are born from the clock's time
// when a scene first held it (0 for the first scene, so a still at t=5
// shows smoke that has risen for 5 seconds). On a held clock
// (options.time) every emitter has been there since 0.
//
// A changed line: emitters are matched by kind and seed. When a scene
// replaces an emitter's line by another with the same kind and seed
// (dust at a walker's feet: a new place each tick), particles born
// before that moment keep the old line's numbers -- place, rate, wind --
// and live out their lives; the new numbers apply to particles born from
// then on. With the rate unchanged the births go on in step (the same
// numbers n, the same hashes), so nothing pops and nothing restarts. An
// emitter whose line is gone (no line of its kind and seed) stops
// emitting and its particles in the air finish. A rate of 0 emits
// nothing. On a held clock a change redraws the whole emitter from 0
// instead (a still is a function of its scene and its time alone).
// Two lines of one kind and seed in a scene are two emitters with the
// same randomness; which of them a later changed line continues is by
// order, so give emitters of a kind different seeds.
//
// Cost: every particle of every kind is one instance of one quad in one
// draw call (premultiplied blending: a kind covers what is behind it by
// its alpha times how much it covers; smoke, dust and splash cover, fire
// and embers mostly add light and cover a little -- purely added, a
// flame over ground in daylight is white, not orange), a soft round
// sprite made in the fragment shader, sized in tiles and
// turned to the camera in view space, so it is right under the board's
// orthographic camera and a perspective one alike. Buffers are written
// only when a scene changes an emitter or a stopped one has finished.
// Smoke, dust and splash take the brightness and color of the board's
// lights (ctx.lights, read each frame); fire and embers glow by
// themselves, a little brighter than 1 at their hottest so that tone
// mapping keeps them bright. Fog tints the kinds that cover and dims the
// light of those that add; tone mapping and the output color space are
// Three's own chunks.
//
// Caps: an emitter has at most ALIVE (600) particles alive: a rate that
// asks for more is lowered to what fits (ALIVE over the kind's longest
// life: smoke 119 a second, dust 532, splash 642, fire 684, embers 184).
// All emitters together have at most TOTAL (20000) slots, given to
// smoke, dust and splash first, then to fire and embers, in the order
// made: an emitter that does not fit whole draws nothing (a warning on
// the console, once), and count() says how many slots went without
// (particles_over). count() also gives emitters and particles (slots in
// use: about rate times the kind's longest life an emitter, plus those
// of stopped emitters still in the air).
//
// Not done: particles are not sorted by depth (they are soft and thin,
// and the additive kinds are drawn after the others), do not fade where
// they meet geometry, cast no shadows, and a splash does not know where
// the water is: it falls back to the height it came from.

// a kind's number in the shader, its longest life in seconds (the
// shader's LIFE times 1.25, or a splash's longest flight), and whether
// it is drawn with the kinds that add light (after the others)
const KINDS = {
  smoke: { id: 0, life: 5.0, add: false },
  dust: { id: 1, life: 1.125, add: false },
  splash: { id: 2, life: 0.933, add: false },
  fire: { id: 3, life: 0.875, add: true },
  embers: { id: 4, life: 3.25, add: true },
};
const ALIVE = 600, TOTAL = 20000;

const VERTEX = /* glsl */ `
attribute vec4 aPlace;  // x, y (up), z in tiles; rate
attribute vec4 aWind;   // wind x, wind z; seed; period (slots)
attribute vec4 aSpan;   // born from, born before; slot; kind
uniform float uTime;
uniform vec3 uLight;
varying vec2 vAt;
varying vec4 vColor;
varying vec3 vShape;    // how much it adds (1) or covers (0), where the soft edge starts, lobes
varying float vTurn;
#include <fog_pars_vertex>

uint mixBits(uint x) {
  x ^= x >> 16; x *= 0x7feb352du; x ^= x >> 15; x *= 0x846ca68bu; x ^= x >> 16;
  return x;
}
float rnd(uint n, uint seed, uint c) {
  return float(mixBits(n * 747796405u + mixBits(seed + c * 2891336453u)) >> 8) / 16777216.0;
}

void main() {
  float rate = aPlace.w, period = aWind.w;
  int kind = int(aSpan.w + 0.5);
  // the newest particle in this slot: n is slot modulo period
  float newest = floor(uTime * rate);
  float n = newest - mod(newest - aSpan.z, period);
  uint un = uint(max(n, 0.0)), us = uint(abs(aWind.z)) * 5u + uint(kind);
  float r0 = rnd(un, us, 0u), r1 = rnd(un, us, 1u), r2 = rnd(un, us, 2u);
  float r3 = rnd(un, us, 3u), r4 = rnd(un, us, 4u), r5 = rnd(un, us, 5u);
  float born = (n + r0) / rate;
  float age = uTime - born;
  float turn = 6.2831853 * r2;
  vec2 out2 = vec2(cos(turn), sin(turn));
  vec2 wind = aWind.xy;

  float life = 1.0, size = 0.1, alpha = 1.0;
  vec3 d = vec3(0.0), color = vec3(1.0);
  vShape = vec3(0.0, 0.0, 0.0);
  if (kind == 0) {
    // smoke: rises, spreads, fades, and the wind takes it as it climbs
    life = 4.0 * (0.75 + 0.5 * r1);
    float u = age / life;
    d.y = 0.12 + (0.55 + 0.35 * r3) * age;
    d.xz = out2 * (0.04 + 0.11 * r4 * age)
      + 0.05 * age * vec2(sin(age * 1.3 + r5 * 6.28), cos(age * 1.1 + r2 * 6.28))
      + wind * (age - (1.0 - exp(-1.2 * age)) / 1.2);
    size = mix(0.35, 1.5, sqrt(clamp(u, 0.0, 1.0))) * (0.8 + 0.4 * r5);
    alpha = 0.5 * smoothstep(0.0, 0.08, u) * pow(max(1.0 - u, 0.0), 1.2);
    color = mix(vec3(0.2), vec3(0.6), smoothstep(0.0, 0.6, u)) * uLight;
    vShape = vec3(0.0, 0.0, 0.2);
  } else if (kind == 1) {
    // dust: low, short, pale
    life = 0.9 * (0.75 + 0.5 * r1);
    float u = age / life;
    d.y = 0.1 + 0.28 * r3 * (1.0 - exp(-2.5 * age));
    d.xz = out2 * (0.08 + 0.34 * r4) * (1.0 - exp(-3.0 * age)) + wind * age;
    size = mix(0.16, 0.6, clamp(u, 0.0, 1.0)) * (0.8 + 0.4 * r5);
    alpha = 0.5 * smoothstep(0.0, 0.1, u) * max(1.0 - u, 0.0);
    color = vec3(0.86, 0.80, 0.68) * uLight;
    vShape = vec3(0.0, 0.0, 0.15);
  } else if (kind == 2) {
    // splash: a drop thrown up and out, falling back to where it came from
    float up = 2.2 + 2.0 * r3;
    life = up / 4.5;
    float u = age / life;
    d.y = up * age - 4.5 * age * age;
    d.xz = out2 * (0.2 + 0.9 * r4) * age + wind * age * 0.3;
    size = (0.08 + 0.08 * r5) * (1.0 - 0.4 * u);
    alpha = 0.9 * (1.0 - u * u * u);
    color = vec3(0.80, 0.90, 1.0) * uLight * 1.25;
    vShape = vec3(0.0, 0.45, 0.0);
  } else if (kind == 3) {
    // fire: bright, short, drawn in to a tongue as it rises; it flickers
    life = 0.7 * (0.75 + 0.5 * r1);
    float u = age / life;
    d.y = 0.06 + (1.0 + 0.7 * r3) * age;
    d.xz = out2 * 0.17 * r4 * (1.0 - 0.8 * u)
      + 0.04 * vec2(sin(uTime * 9.0 + r5 * 6.28), cos(uTime * 7.0 + r2 * 6.28)) * u
      + wind * age * 0.6;
    size = mix(0.4, 0.1, clamp(u, 0.0, 1.0)) * (0.8 + 0.4 * r5);
    float flicker = 0.8 + 0.2 * sin(uTime * 13.0 + aWind.z) * sin(uTime * 7.3 + r5 * 6.28);
    alpha = 0.5 * smoothstep(0.0, 0.12, u) * max(1.0 - u, 0.0) * flicker;
    color = mix(vec3(1.5, 0.62, 0.09), vec3(1.0, 0.13, 0.01), smoothstep(0.0, 0.6, u));
    vShape = vec3(0.6, 0.0, 0.0);
  } else {
    // embers: a few bright points that drift up, twinkle and die
    life = 2.6 * (0.75 + 0.5 * r1);
    float u = age / life;
    d.y = 0.3 + (0.55 + 0.6 * r3) * age;
    d.xz = out2 * 0.15 * r4 * age
      + 0.12 * vec2(sin(age * 3.0 + r5 * 6.28), cos(age * 2.3 + r2 * 6.28)) * min(age, 1.0)
      + wind * age * 0.9;
    size = 0.07 + 0.05 * r5;
    float twinkle = 0.65 + 0.35 * sin(uTime * 17.0 + r2 * 40.0);
    alpha = (1.0 - u * u) * twinkle;
    color = mix(vec3(1.6, 0.55, 0.06), vec3(1.2, 0.15, 0.01), u);
    vShape = vec3(0.5, 0.15, 0.0);
  }

  vAt = position.xy * 2.0;
  vTurn = turn;
  vColor = vec4(color, alpha);
  bool live = n >= 0.0 && age >= 0.0 && age < life && born >= aSpan.x && born < aSpan.y;
  if (!live) {
    gl_Position = vec4(2.0, 2.0, 2.0, 1.0);
    return;
  }
  // the quad faces the camera: its corners are set out in view space, in tiles
  vec4 mvPosition = modelViewMatrix * vec4(aPlace.xyz + d, 1.0);
  mvPosition.xy += position.xy * size;
  gl_Position = projectionMatrix * mvPosition;
  #include <fog_vertex>
}
`;

const FRAGMENT = /* glsl */ `
uniform vec3 fogColor;
uniform float fogDensity;
uniform float fogNear;
uniform float fogFar;
varying vec2 vAt;
varying vec4 vColor;
varying vec3 vShape;
varying float vTurn;
#ifdef USE_FOG
  varying float vFogDepth;
#endif

void main() {
  // a soft round sprite; smoke and dust have a few lobes, so a puff is not a disk
  float r = length(vAt);
  r *= 1.0 + vShape.z * sin(3.0 * atan(vAt.y, vAt.x) + vTurn);
  if (r >= 1.0) discard;
  float soft = 1.0 - smoothstep(vShape.y, 1.0, r);
  float alpha = vColor.a * soft * soft;
  gl_FragColor = vec4(vColor.rgb, alpha);
  #include <tonemapping_fragment>
  #include <colorspace_fragment>
  #ifdef USE_FOG
    #ifdef FOG_EXP2
      float fogFactor = 1.0 - exp(-fogDensity * fogDensity * vFogDepth * vFogDepth);
    #else
      float fogFactor = smoothstep(fogNear, fogFar, vFogDepth);
    #endif
    // fog hides light that is added, and tints what covers
    gl_FragColor.rgb = mix(gl_FragColor.rgb, fogColor, fogFactor * (1.0 - vShape.x));
    alpha *= 1.0 - fogFactor * vShape.x;
  #endif
  // premultiplied: what adds covers less
  gl_FragColor = vec4(gl_FragColor.rgb * alpha, alpha * (1.0 - vShape.x));
}
`;

export default function particles(ctx) {
  const { THREE, scene, options } = ctx;
  const held = options.time !== undefined;
  const segments = [];       // what is drawn: an emitter's numbers over a span of births
  const leaving = new Map(); // kind and seed -> segments whose lines are gone this scene
  const uniforms = THREE.UniformsUtils.merge([THREE.UniformsLib.fog, { uTime: { value: 0 }, uLight: { value: new THREE.Color(1, 1, 1) } }]);
  const quad = new THREE.PlaneGeometry(1, 1);
  let clock = held ? options.time : 0, mesh = null, material = null, room = 0, used = 0, over = 0;
  let dirty = false, expires = Infinity, stopped = false, warned = false, made = 0;
  let place = null, wind = null, span = null;

  // the buffers, with room for at least n slots
  const fit = (n) => {
    if (mesh && n <= room) return;
    room = Math.min(TOTAL, Math.max(1024, 2 * n));
    const geometry = new THREE.InstancedBufferGeometry();
    geometry.index = quad.index;
    geometry.setAttribute('position', quad.getAttribute('position'));
    const attribute = () => new THREE.InstancedBufferAttribute(new Float32Array(room * 4), 4).setUsage(THREE.DynamicDrawUsage);
    place = attribute(); wind = attribute(); span = attribute();
    geometry.setAttribute('aPlace', place);
    geometry.setAttribute('aWind', wind);
    geometry.setAttribute('aSpan', span);
    if (mesh) {
      mesh.geometry.dispose();
      mesh.geometry = geometry;
      return;
    }
    material = new THREE.ShaderMaterial({
      uniforms, vertexShader: VERTEX, fragmentShader: FRAGMENT, fog: true, transparent: true, depthWrite: false,
      blending: THREE.CustomBlending, blendEquation: THREE.AddEquation, blendSrc: THREE.OneFactor, blendDst: THREE.OneMinusSrcAlphaFactor,
    });
    mesh = new THREE.Mesh(geometry, material);
    mesh.name = 'particles';
    mesh.frustumCulled = false;
    mesh.renderOrder = 10;
    scene.add(mesh);
  };

  // every segment's slots written out: the kinds that cover, then the kinds that add
  const write = () => {
    dirty = false;
    for (let i = segments.length - 1; i >= 0; i--) if (!segments[i].open && segments[i].until <= clock) segments.splice(i, 1);
    expires = Infinity;
    const order = segments.filter((s) => !s.kind.add).concat(segments.filter((s) => s.kind.add));
    let want = 0;
    for (const s of order) {
      // an emitter still emitting fills its period; a stopped one keeps the
      // slots of the particles born in its span (a little more: the shader decides)
      s.first = s.open ? 0 : Math.floor(s.from * s.rate) - 2;
      s.slots = s.open ? s.period : Math.min(s.period, Math.floor(s.to * s.rate) + 1 - s.first + 1);
      want += s.slots;
      if (!s.open) expires = Math.min(expires, s.until);
    }
    used = 0; over = 0;
    if (!want && !mesh) return;
    fit(Math.min(want, TOTAL));
    const a = place.array, b = wind.array, c = span.array;
    for (const s of order) {
      if (used + s.slots > TOTAL) { over += s.slots; continue; }
      for (let j = 0; j < s.slots; j++, used++) {
        const o = used * 4;
        a[o] = s.place[0]; a[o + 1] = s.place[2]; a[o + 2] = s.place[1]; a[o + 3] = s.rate;
        b[o] = s.wind[0]; b[o + 1] = s.wind[1]; b[o + 2] = s.seed; b[o + 3] = s.period;
        c[o] = s.from; c[o + 1] = s.open ? 1e30 : s.to; c[o + 2] = (((s.first + j) % s.period) + s.period) % s.period; c[o + 3] = s.kind.id;
      }
    }
    if (over && !warned) { warned = true; console.warn(`particles: more than ${TOTAL} in all; the emitters made last are not drawn`); }
    for (const x of [place, wind, span]) { x.clearUpdateRanges(); x.addUpdateRange(0, used * 4); x.needsUpdate = true; }
    mesh.geometry.instanceCount = used;
    mesh.visible = used > 0;
  };

  // an emitter stops: what it has in the air finishes (on a held clock there is no before)
  const close = (s) => {
    dirty = true;
    s.open = false;
    s.to = clock;
    s.until = clock + s.kind.life;
    if (held || s.to <= s.from) segments.splice(segments.indexOf(s), 1);
  };

  const emitter = (line, name, at, rate, blows, seed) => {
    const kind = KINDS[name];
    if (!kind) { console.warn(`particles: no kind '${name}' (smoke, dust, splash, fire, embers)`); return {}; }
    const key = `${name} ${seed}`;
    // the emitter this line goes on from, when the scene before had one
    const before = leaving.get(key);
    if (before && before.length) close(before.shift());
    if (!(rate > 0)) return {};
    const most = (ALIVE - 1) / kind.life;
    const s = {
      kind, key, place: [at[0] || 0, at[1] || 0, at[2] || 0], wind: [(blows && blows[0]) || 0, (blows && blows[1]) || 0], seed: Number(seed) || 0,
      rate: Math.min(rate, most), from: held ? 0 : clock, to: Infinity, until: Infinity, open: true, order: made++, first: 0, slots: 0, period: 0,
    };
    s.period = Math.ceil(s.rate * kind.life) + 1;
    segments.push(s);
    dirty = true;
    return {
      dispose() {
        if (stopped || !s.open) return;
        if (!leaving.has(key)) leaving.set(key, []);
        leaving.get(key).push(s);
      },
    };
  };

  // the light smoke, dust and water are seen by: the board's sky and sun, as a
  // white surface facing up would take them
  const sunward = new THREE.Color(), above = new THREE.Color();
  const lit = () => {
    const { sky, sun } = ctx.lights || {}, light = uniforms.uLight.value;
    if (!sky && !sun) { light.setRGB(1, 1, 1); return; }
    light.setRGB(0, 0, 0);
    if (sky && sky.visible !== false && sky.color) {
      above.copy(sky.color);
      if (sky.groundColor) above.lerp(sky.groundColor, 0.15);
      light.add(above.multiplyScalar(sky.intensity / Math.PI));
    }
    if (sun && sun.visible !== false && sun.color) light.add(sunward.copy(sun.color).multiplyScalar(sun.intensity * 0.7 / Math.PI));
  };

  return {
    kept: { particles: emitter },
    // lines gone and not gone on from: those emitters stop
    end() {
      for (const list of leaving.values()) for (const s of list) close(s);
      leaving.clear();
    },
    frame(time) {
      clock = time;
      if (dirty || clock >= expires) write();
      if (!mesh) return;
      uniforms.uTime.value = clock;
      lit();
    },
    count() {
      return { emitters: segments.filter((s) => s.open).length, particles: used, particles_over: over };
    },
    stop() {
      stopped = true;
      if (mesh) { scene.remove(mesh); mesh.geometry.dispose(); material.dispose(); }
      quad.dispose();
    },
  };
}
