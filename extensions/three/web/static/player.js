// The Three.js player: the functions a scene's calls use (the same
// calls Cycles.xtl writes for Blender's player, cy:head), drawn in real
// time. It reads the numbers and makes the objects, materials and
// motion, and decides nothing about the scene. Meters, z up, seconds.
import * as THREE from 'three';
import { EffectComposer } from './vendor/three/postprocessing/EffectComposer.js';
import { RenderPass } from './vendor/three/postprocessing/RenderPass.js';
import { UnrealBloomPass } from './vendor/three/postprocessing/UnrealBloomPass.js';
import { OutputPass } from './vendor/three/postprocessing/OutputPass.js';

const COMMON = `
uniform vec3 uFogColor;
uniform float uFogDensity;
uniform float uTime;
uniform vec3 uLampPos[32];
uniform vec3 uLampCol;
uniform int uLampCount;
uniform vec3 uAmbient;
float hash3(vec3 p) {
  p = fract(p * 0.1031);
  p += dot(p, p.yzx + 33.33);
  return fract((p.x + p.y) * p.z);
}
float vnoise(vec3 p) {
  vec3 i = floor(p), f = fract(p);
  f = f * f * (3.0 - 2.0 * f);
  float a = mix(mix(hash3(i), hash3(i + vec3(1, 0, 0)), f.x), mix(hash3(i + vec3(0, 1, 0)), hash3(i + vec3(1, 1, 0)), f.x), f.y);
  float b = mix(mix(hash3(i + vec3(0, 0, 1)), hash3(i + vec3(1, 0, 1)), f.x), mix(hash3(i + vec3(0, 1, 1)), hash3(i + vec3(1, 1, 1)), f.x), f.y);
  return mix(a, b, f.z);
}
float fbm(vec3 p) {
  float s = 0.0, a = 0.5;
  for (int i = 0; i < 4; i++) { s += a * vnoise(p); p *= 2.03; a *= 0.5; }
  return s / 0.9375;
}
vec3 lamps(vec3 wp, vec3 n) {
  vec3 s = vec3(0.0);
  for (int i = 0; i < 32; i++) {
    if (i >= uLampCount) break;
    vec3 l = uLampPos[i] - wp;
    float d2 = dot(l, l);
    if (d2 < 2500.0) s += max(dot(n, normalize(l)), 0.0) / (d2 + 1.0);
  }
  return s * uLampCol;
}
vec3 fogged(vec3 c, vec3 wp) {
  return mix(uFogColor, c, exp(-uFogDensity * distance(cameraPosition, wp)));
}
`;

const VERT = `
varying vec3 vPos;
varying vec3 vN;
varying vec2 vUv;
void main() {
  vec4 w = modelMatrix * vec4(position, 1.0);
  vPos = w.xyz;
  vN = normalize(mat3(modelMatrix) * normal);
  vUv = uv;
  gl_Position = projectionMatrix * viewMatrix * w;
}
`;

const FRAG = {
  surface: `
uniform vec3 uColor;
uniform float uRough;
uniform float uMetal;
varying vec3 vPos;
varying vec3 vN;
void main() {
  vec3 n = normalize(vN);
  vec3 v = normalize(cameraPosition - vPos);
  float rim = pow(1.0 - max(dot(n, v), 0.0), 3.0) * (1.0 - uRough);
  vec3 c = uColor * (uAmbient * (0.6 + 0.4 * n.z) + lamps(vPos, n)) + rim * (0.3 + uMetal) * uAmbient;
  gl_FragColor = vec4(fogged(c, vPos), 1.0);
}`,
  neon: `
uniform vec3 uColor;
uniform float uStrength;
varying vec3 vPos;
void main() {
  gl_FragColor = vec4(fogged(uColor * uStrength, vPos), 1.0);
}`,
  facade: `
uniform vec3 uWall;
uniform vec3 uLitA;
uniform vec3 uLitB;
uniform vec2 uCell;
uniform float uLit;
uniform float uGlow;
varying vec3 vPos;
varying vec3 vN;
void main() {
  vec3 n = normalize(vN);
  float ax = step(0.5, abs(n.x));
  float roof = step(0.5, abs(n.z));
  float u = mix(vPos.x, vPos.y, ax);
  float depth = mix(vPos.y, vPos.x, ax);
  vec2 c = vec2(u / uCell.x, vPos.z / uCell.y);
  vec2 i = floor(c), f = fract(c);
  vec2 w = fwidth(c);
  float side = floor(depth * 2.0 + 0.5) + ax * 977.0;
  float pane = smoothstep(0.14 - w.x, 0.14 + w.x, f.x) * smoothstep(0.86 + w.x, 0.86 - w.x, f.x)
             * smoothstep(0.3 - w.y, 0.3 + w.y, f.y) * smoothstep(0.88 + w.y, 0.88 - w.y, f.y) * (1.0 - roof);
  vec3 cell = vec3(i, side);
  float r = hash3(cell);
  float fl = hash3(vec3(0.5, i.y, side) + 7.1);
  float on = pane * step(r, uLit + (fl - 0.5) * 0.5);
  vec3 room = mix(uLitA, uLitB, hash3(cell + 3.7)) * (0.25 + hash3(cell + 9.2));
  float far = smoothstep(0.35, 0.9, max(w.x, w.y));
  vec3 lit = mix(on * room, (1.0 - roof) * uLit * 0.42 * 0.75 * mix(uLitA, uLitB, 0.5), far) * uGlow;
  float grime = fbm(vec3(vPos.xy * 0.6, vPos.z * 0.08));
  vec3 base = mix(uWall * (0.45 + 0.8 * grime), vec3(0.013, 0.014, 0.02), pane * (1.0 - far));
  vec3 v = normalize(cameraPosition - vPos);
  float sheen = pow(1.0 - max(dot(n, v), 0.0), 4.0) * pane;
  vec3 col = base * (uAmbient * (0.7 + 0.3 * n.z) + lamps(vPos, n)) + sheen * uAmbient * 0.6 + lit;
  gl_FragColor = vec4(fogged(col, vPos), 1.0);
}`,
  screen: `
uniform int uKind;
uniform vec3 uA;
uniform vec3 uB;
uniform float uSpeed;
uniform float uStrength;
varying vec3 vPos;
varying vec2 vUv;
void main() {
  float t = uTime * uSpeed, f, level;
  float u = vUv.x, v = vUv.y;
  if (uKind == 0) {
    f = smoothstep(-0.2, 0.2, sin((v * 5.0 + t) * 6.2832));
    level = 0.35 + 0.65 * f;
  } else if (uKind == 1) {
    float d = length(vec2(u, v) - 0.5);
    f = smoothstep(-0.3, 0.3, sin((d * 7.0 - t) * 6.2832));
    level = 0.3 + 0.7 * f;
  } else if (uKind == 2) {
    float col = floor(u * 28.0);
    float r2 = hash3(vec3(col, 3.0, 7.0)), r3 = hash3(vec3(col, 5.0, 1.0));
    float fall = fract(v * 1.5 + t * (0.4 + r2) + r3);
    float glyph = hash3(vec3(col, floor(v * 44.0), floor(t * 6.0)));
    f = pow(fall, 6.0);
    level = pow(fall, 3.0) * (0.3 + glyph);
  } else {
    vec3 cell = vec3(floor(u * 6.0), floor(v * 4.0), floor(t));
    f = hash3(cell);
    level = 0.15 + 0.85 * hash3(cell + 4.4);
  }
  float lines = 0.8 + 0.2 * sin(v * 400.0);
  gl_FragColor = vec4(fogged(mix(uA, uB, f) * uStrength * level * lines, vPos), 1.0);
}`,
  wet: `
uniform vec3 uColor;
uniform float uPuddles;
uniform float uSize;
uniform float uRipple;
uniform sampler2D uMirror;
uniform mat4 uMirrorMatrix;
varying vec3 vPos;
void main() {
  vec3 p = vec3(vPos.xy, 0.0);
  float big = fbm(p / uSize + 11.0);
  float th = 0.5 + (0.5 - uPuddles) * 0.5;
  float pool = smoothstep(th - 0.03, th + 0.03, big);
  float fine = fbm(p * 2.5);
  vec2 rip = vec2(vnoise(vec3(p.xy * 5.0, uTime * 2.5)), vnoise(vec3(p.yx * 5.0 + 9.0, uTime * 2.5))) - 0.5;
  vec2 bump = mix((vec2(fbm(p * 3.0), fbm(p * 3.0 + 5.0)) - 0.5) * 0.05, rip * uRipple * 0.03, pool);
  vec3 n = normalize(vec3(bump, 1.0));
  vec3 v = normalize(cameraPosition - vPos);
  vec4 m = uMirrorMatrix * vec4(vPos.xy + bump * 8.0, 0.0, 1.0);
  vec3 mirror = texture2D(uMirror, m.xy / m.w).rgb;
  float fres = 0.04 + 0.96 * pow(1.0 - max(dot(n, v), 0.0), 5.0);
  vec3 dry = uColor * (0.5 + 0.8 * fine) * (uAmbient + lamps(vPos, n));
  vec3 damp = dry + mirror * fres * 0.14;
  vec3 water = uColor * 0.25 * uAmbient + mirror * min(1.0, fres * 1.6 + 0.12);
  gl_FragColor = vec4(fogged(mix(damp, water, pool), vPos), 1.0);
}`,
  sign: `
uniform vec3 uColor;
uniform float uStrength;
uniform sampler2D uMap;
varying vec3 vPos;
varying vec2 vUv;
void main() {
  float a = texture2D(uMap, vUv).a;
  if (a < 0.5) discard;
  gl_FragColor = vec4(fogged(uColor * uStrength, vPos), 1.0);
}`,
  sky: `
uniform vec3 uHorizon;
uniform vec3 uZenith;
varying vec3 vDir;
void main() {
  float z = abs(normalize(vDir).z);
  gl_FragColor = vec4(mix(mix(uFogColor, uHorizon, 0.5), uZenith, smoothstep(0.0, 0.6, z)), 1.0);
}`,
  steam: `
uniform vec3 uColor;
uniform float uDensity;
uniform float uSpeed;
uniform float uHeight;
uniform float uSeed;
varying vec2 vUv;
varying vec3 vPos;
void main() {
  float up = vUv.y;
  float wide = 0.12 + up * 0.38;
  float core = 1.0 - smoothstep(0.3, 1.0, abs(vUv.x - 0.5) / wide);
  float fade = smoothstep(0.0, 0.08, up) * (1.0 - smoothstep(0.35, 1.0, up));
  float puff = fbm(vec3(vUv.x * 3.0 + uSeed, (up * uHeight - uTime * uSpeed) * 0.45, uTime * 0.25 + uSeed));
  float a = core * fade * smoothstep(0.3, 0.7, puff) * min(1.0, uDensity * 0.55);
  vec3 lit = uColor * (uAmbient * 2.5 + lamps(vPos, vec3(0.0, 0.0, 1.0)) * 6.0 + 0.05);
  gl_FragColor = vec4(fogged(lit, vPos), a);
}`,
};

const SKY_VERT = `
varying vec3 vDir;
void main() {
  vDir = position;
  vec4 p = projectionMatrix * viewMatrix * vec4(position + cameraPosition, 1.0);
  gl_Position = p.xyww;
}`;

const STEAM_VERT = `
uniform vec3 uAt;
uniform vec2 uExtent;
varying vec2 vUv;
varying vec3 vPos;
void main() {
  vUv = uv;
  vec3 to = cameraPosition - uAt;
  vec3 right = normalize(vec3(-to.y, to.x, 0.0) + vec3(1e-4, 0.0, 0.0));
  vPos = uAt + right * (uv.x - 0.5) * uExtent.x + vec3(0.0, 0.0, (uv.y - 0.5) * uExtent.y);
  gl_Position = projectionMatrix * viewMatrix * vec4(vPos, 1.0);
}`;

const FACES = [[0, 1, 5, 4], [3, 2, 6, 7], [1, 3, 7, 5], [2, 0, 4, 6], [4, 5, 7, 6], [0, 2, 3, 1]];
const NORMALS = [[0, -1, 0], [0, 1, 0], [1, 0, 0], [-1, 0, 0], [0, 0, 1], [0, 0, -1]];

// Draws the scene the calls describe into host (an element); options:
// time (seconds: hold the flight there), mode ('ride', 'fly', 'walk'),
// pixels (a cap on the device pixel ratio). Returns its controls.
export function play(host, calls, options = {}) {
  const shared = {
    uFogColor: { value: new THREE.Color(0, 0, 0) },
    uFogDensity: { value: 0 },
    uTime: { value: 0 },
    uLampPos: { value: Array.from({ length: 32 }, () => new THREE.Vector3(0, 0, -1e4)) },
    uLampCol: { value: new THREE.Color(0, 0, 0) },
    uLampCount: { value: 0 },
    uAmbient: { value: new THREE.Color(0.05, 0.05, 0.08) },
  };
  const mirror = { uMirror: { value: null }, uMirrorMatrix: { value: new THREE.Matrix4() } };
  const scene = new THREE.Scene();
  const camera = new THREE.PerspectiveCamera(50, 16 / 9, 0.1, 5000);
  camera.up.set(0, 0, 1);
  const mats = {}, models = {}, movers = [], solids = [], grounds = [], counts = {};
  const film = { seconds: 15, times: [0, 1], eyes: [[0, -10, 5], [0, 0, 5]], aims: [[0, 0, 5], [0, 10, 5]], exposure: 0, bloom: null };

  const shader = (frag, uniforms, extra = {}) => new THREE.ShaderMaterial({
    uniforms: { ...shared, ...uniforms }, vertexShader: VERT, fragmentShader: COMMON + frag, ...extra,
  });
  const color = (rgb) => new THREE.Color(rgb[0], rgb[1], rgb[2]);
  const count = (kind, n = 1) => { counts[kind] = (counts[kind] || 0) + n; };

  const api = {
    nums(text, k) {
      const v = text.trim().split(/\s+/).map(Number), rows = [];
      for (let i = 0; i < v.length; i += k) rows.push(v.slice(i, i + k));
      return rows;
    },
    stage(w, h, fps, seconds) { film.seconds = seconds; },
    look(name, exposure) { film.exposure = exposure; },
    sky(horizon, zenith, strength) {
      const m = new THREE.ShaderMaterial({
        uniforms: { ...shared, uHorizon: { value: color(horizon).multiplyScalar(strength) }, uZenith: { value: color(zenith).multiplyScalar(strength) } },
        vertexShader: SKY_VERT, fragmentShader: COMMON + FRAG.sky, side: THREE.BackSide, depthWrite: false,
      });
      const dome = new THREE.Mesh(new THREE.SphereGeometry(1, 16, 8), m);
      dome.frustumCulled = false;
      dome.renderOrder = -1;
      scene.add(dome);
    },
    bloom(threshold, strength, size) { film.bloom = [threshold, strength, size]; },
    surface(name, rgb, roughness, metallic) {
      mats[name] = shader(FRAG.surface, { uColor: { value: color(rgb) }, uRough: { value: roughness }, uMetal: { value: metallic } });
    },
    neon(name, rgb, strength) {
      mats[name] = shader(FRAG.neon, { uColor: { value: color(rgb) }, uStrength: { value: strength } });
    },
    facade(name, wall, a, b, cell, lit, glow) {
      mats[name] = shader(FRAG.facade, {
        uWall: { value: color(wall) }, uLitA: { value: color(a) }, uLitB: { value: color(b) },
        uCell: { value: new THREE.Vector2(cell[0], cell[1]) }, uLit: { value: lit }, uGlow: { value: glow },
      });
    },
    wet(name, rgb, puddles, size, ripple) {
      mats[name] = shader(FRAG.wet, { ...mirror, uColor: { value: color(rgb) }, uPuddles: { value: puddles }, uSize: { value: size }, uRipple: { value: ripple } });
      mats[name].userData.ground = true;
    },
    screen(name, kind, a, b, speed, strength) {
      mats[name] = shader(FRAG.screen, {
        uKind: { value: Math.max(0, ['bars', 'rings', 'rain', 'tiles'].indexOf(kind)) }, uA: { value: color(a) }, uB: { value: color(b) },
        uSpeed: { value: speed }, uStrength: { value: strength },
      });
    },
    reel(name, path, strength) { api.screen(name, 'tiles', [1, 0.2, 0.4], [0.2, 0.6, 1], 1.0, strength); },
    boxes(name, rows, material, into = '') {
      const pos = [], nor = [], uvs = [], idx = [];
      for (const r of rows) {
        const ca = Math.cos(r[6] * Math.PI / 180), sa = Math.sin(r[6] * Math.PI / 180), c = [];
        for (let i = 0; i < 8; i++) {
          const x = ((i & 1) - 0.5) * r[3], y = ((i >> 1 & 1) - 0.5) * r[4], z = ((i >> 2) - 0.5) * r[5];
          c.push([r[0] + x * ca - y * sa, r[1] + x * sa + y * ca, r[2] + z]);
        }
        FACES.forEach((f, k) => {
          const base = pos.length / 3, n = NORMALS[k];
          f.forEach((v, j) => {
            pos.push(...c[v]);
            nor.push(n[0] * ca - n[1] * sa, n[0] * sa + n[1] * ca, n[2]);
            uvs.push(j === 1 || j === 2 ? 1 : 0, j >= 2 ? 1 : 0);
          });
          idx.push(base, base + 1, base + 2, base, base + 2, base + 3);
        });
        // what a flier or a walker cannot pass through: a standing box
        // of some size (not the ground, a band, a screen or a slab)
        if (!into && r[6] === 0 && Math.min(r[3], r[4]) > 0.9 && r[5] > 1.0 && r[2] + r[5] / 2 > 1.0) {
          solids.push([r[0] - r[3] / 2, r[1] - r[4] / 2, r[2] - r[5] / 2, r[0] + r[3] / 2, r[1] + r[4] / 2, r[2] + r[5] / 2]);
        }
      }
      const g = new THREE.BufferGeometry();
      g.setAttribute('position', new THREE.Float32BufferAttribute(pos, 3));
      g.setAttribute('normal', new THREE.Float32BufferAttribute(nor, 3));
      g.setAttribute('uv', new THREE.Float32BufferAttribute(uvs, 2));
      g.setIndex(idx);
      const mesh = new THREE.Mesh(g, mats[material]);
      mesh.name = name;
      count('boxes', rows.length);
      if (into) { models[into].push(mesh); return; }
      if (mats[material].userData.ground) grounds.push(mesh);
      scene.add(mesh);
    },
    sign(name, body, at, size, turn, stack, material) {
      const letters = stack ? body.split('') : [body], px = 96;
      const cv = document.createElement('canvas'), cx = cv.getContext('2d');
      cx.font = `900 ${px}px ui-monospace, Menlo, Consolas, monospace`;
      const wide = Math.ceil(Math.max(...letters.map((l) => cx.measureText(l).width))) + 16;
      cv.width = wide;
      cv.height = Math.ceil(px * 1.05 * letters.length) + 8;
      cx.font = `900 ${px}px ui-monospace, Menlo, Consolas, monospace`;
      cx.textAlign = 'center';
      cx.textBaseline = 'middle';
      cx.fillStyle = '#fff';
      letters.forEach((l, i) => cx.fillText(l, wide / 2, 4 + px * 1.05 * (i + 0.5)));
      const map = new THREE.CanvasTexture(cv);
      const src = mats[material].uniforms;
      const m = shader(FRAG.sign, { uColor: src.uColor, uStrength: src.uStrength, uMap: { value: map } }, { side: THREE.DoubleSide });
      const scale = size * 1.25 / px, w = cv.width * scale, h = cv.height * scale;
      const g = new THREE.PlaneGeometry(w, h);
      const mesh = new THREE.Mesh(g, m);
      mesh.name = name;
      mesh.rotation.set(Math.PI / 2, 0, turn * Math.PI / 180, 'ZXY');
      mesh.position.set(at[0], at[1], at[2] - (stack ? h / 2 : 0));
      count('signs');
      scene.add(mesh);
    },
    model(name) { models[name] = []; },
    fleet(name, kind, rows) {
      for (const r of rows) {
        const g = new THREE.Group();
        for (const part of models[kind]) g.add(new THREE.Mesh(part.geometry, part.material));
        g.rotation.z = Math.atan2(r[4] - r[1], r[3] - r[0]);
        scene.add(g);
        movers.push({ g, from: r.slice(0, 3), to: r.slice(3, 6) });
      }
      count('movers', rows.length);
    },
    lamps(name, rows) {
      for (const r of rows) {
        const i = shared.uLampCount.value;
        if (i >= 32) break;
        shared.uLampPos.value[i].set(r[0], r[1], r[2]);
        shared.uLampCol.value.setRGB(r[3], r[4], r[5]).multiplyScalar(r[6] * 0.012);
        shared.uLampCount.value = i + 1;
      }
      count('lamps', rows.length);
    },
    // spotlights are not drawn (the city has none)
    spots() {},
    fog(name, rows, rgb, density) {
      shared.uFogDensity.value = density;
      shared.uFogColor.value.setRGB(rgb[0], rgb[1], rgb[2]).multiplyScalar(0.016);
      shared.uAmbient.value.setRGB(rgb[0], rgb[1], rgb[2]).multiplyScalar(0.3);
    },
    steam(name, rows, rgb, density, speed) {
      rows.forEach((r, i) => {
        const m = new THREE.ShaderMaterial({
          uniforms: {
            ...shared, uColor: { value: color(rgb) }, uDensity: { value: density }, uSpeed: { value: speed }, uHeight: { value: r[5] },
            uSeed: { value: i * 7.3 }, uAt: { value: new THREE.Vector3(r[0], r[1], r[2]) }, uExtent: { value: new THREE.Vector2(r[3] * 1.4, r[5]) },
          },
          vertexShader: STEAM_VERT, fragmentShader: COMMON + FRAG.steam, transparent: true, depthWrite: false, side: THREE.DoubleSide,
        });
        const mesh = new THREE.Mesh(new THREE.PlaneGeometry(1, 1), m);
        mesh.frustumCulled = false;
        mesh.renderOrder = 2;
        scene.add(mesh);
      });
      count('plumes', rows.length);
    },
    fly(times, eyes, aims, lens) {
      film.times = times; film.eyes = eyes; film.aims = aims;
      camera.fov = 2 * Math.atan(12 / lens) * 180 / Math.PI;
    },
    // rendering to files is the film's: nothing to do here
    still() {},
    frames() {},
    encode() {},
    report() { api.reported = { ...counts, materials: Object.keys(mats).sort() }; },
  };
  new Function(...Object.keys(api), calls)(...Object.values(api));

  // the picture
  const renderer = new THREE.WebGLRenderer({ antialias: true, powerPreference: 'high-performance', preserveDrawingBuffer: !!options.keep });
  renderer.setPixelRatio(Math.min(window.devicePixelRatio || 1, options.pixels || 1.5));
  renderer.toneMapping = THREE.AgXToneMapping;
  renderer.toneMappingExposure = 2 ** film.exposure;
  host.appendChild(renderer.domElement);
  renderer.domElement.style.display = 'block';
  const target = new THREE.WebGLRenderTarget(4, 4, { type: THREE.HalfFloatType });
  mirror.uMirror.value = target.texture;
  const below = new THREE.PerspectiveCamera();
  below.up.set(0, 0, -1);
  const composer = new EffectComposer(renderer);
  composer.addPass(new RenderPass(scene, camera));
  if (film.bloom) composer.addPass(new UnrealBloomPass(new THREE.Vector2(4, 4), film.bloom[1] * 0.45, film.bloom[2] * 0.5, film.bloom[0]));
  composer.addPass(new OutputPass());
  const size = () => {
    const w = host.clientWidth || 1280, h = host.clientHeight || 720;
    renderer.setSize(w, h);
    composer.setSize(w, h);
    const r = renderer.getPixelRatio();
    target.setSize(Math.ceil(w * r / 2), Math.ceil(h * r / 2));
    camera.aspect = w / h;
    camera.updateProjectionMatrix();
  };
  size();
  const gone = new AbortController(), until = { signal: gone.signal };
  window.addEventListener('resize', size, until);

  // the film's flight: a smooth curve through the eyes and the aims
  const along = (points, t) => {
    const ts = film.times, n = ts.length;
    let i = 0;
    while (i < n - 2 && ts[i + 1] <= t) i++;
    const u = Math.min(1, Math.max(0, (t - ts[i]) / (ts[i + 1] - ts[i])));
    const p = (d) => points[Math.max(0, Math.min(n - 1, i + d))];
    const [a, b, c, d] = [p(-1), p(0), p(1), p(2)];
    return [0, 1, 2].map((k) => 0.5 * (2 * b[k] + (c[k] - a[k]) * u + (2 * a[k] - 5 * b[k] + 4 * c[k] - d[k]) * u * u + (3 * b[k] - a[k] - 3 * c[k] + d[k]) * u ** 3));
  };

  // flying and walking
  const state = { mode: options.mode || 'ride', yaw: Math.PI / 2, pitch: 0, keys: new Set(), time: options.time || 0, held: options.time !== undefined };
  const eye = new THREE.Vector3(0, -150, 30);
  const facing = () => new THREE.Vector3(Math.cos(state.yaw) * Math.cos(state.pitch), Math.sin(state.yaw) * Math.cos(state.pitch), Math.sin(state.pitch));
  const leave = (mode) => {
    if (state.mode === 'ride') {
      const d = new THREE.Vector3();
      camera.getWorldDirection(d);
      eye.copy(camera.position);
      state.yaw = Math.atan2(d.y, d.x);
      state.pitch = Math.asin(Math.max(-1, Math.min(1, d.z)));
    }
    state.mode = mode;
    if (mode === 'walk') eye.z = 1.7;
    if (options.onMode) options.onMode(mode);
  };
  const el = renderer.domElement;
  el.tabIndex = 0;
  el.addEventListener('click', () => { el.focus(); if (state.mode !== 'ride' && el.requestPointerLock) el.requestPointerLock(); }, until);
  document.addEventListener('mousemove', (e) => {
    if (document.pointerLockElement !== el || state.mode === 'ride') return;
    state.yaw -= e.movementX * 0.0022;
    state.pitch = Math.max(-1.5, Math.min(1.5, state.pitch - e.movementY * 0.0022));
  }, until);
  const moving = ['KeyW', 'KeyA', 'KeyS', 'KeyD', 'KeyQ', 'KeyE', 'Space', 'ArrowUp', 'ArrowDown', 'ArrowLeft', 'ArrowRight'];
  window.addEventListener('keydown', (e) => {
    if (e.target && /INPUT|TEXTAREA|SELECT/.test(e.target.tagName)) return;
    if (moving.includes(e.code)) {
      if (state.mode === 'ride') leave('fly');
      state.keys.add(e.code);
      e.preventDefault();
    }
    if (e.code === 'ShiftLeft' || e.code === 'ShiftRight') state.keys.add('Shift');
  }, until);
  window.addEventListener('keyup', (e) => { state.keys.delete(e.code); if (e.code.startsWith('Shift')) state.keys.delete('Shift'); }, until);
  const step = (dt) => {
    const k = state.keys, f = facing();
    if (k.has('ArrowLeft')) state.yaw += dt * 1.6;
    if (k.has('ArrowRight')) state.yaw -= dt * 1.6;
    const walk = state.mode === 'walk';
    const ahead = walk ? new THREE.Vector3(Math.cos(state.yaw), Math.sin(state.yaw), 0) : f;
    const right = new THREE.Vector3(Math.sin(state.yaw), -Math.cos(state.yaw), 0);
    const v = new THREE.Vector3();
    if (k.has('KeyW') || k.has('ArrowUp')) v.add(ahead);
    if (k.has('KeyS') || k.has('ArrowDown')) v.sub(ahead);
    if (k.has('KeyD')) v.add(right);
    if (k.has('KeyA')) v.sub(right);
    if (!walk && (k.has('KeyE') || k.has('Space'))) v.z += 1;
    if (!walk && k.has('KeyQ')) v.z -= 1;
    const speed = (walk ? 5 : 28) * (k.has('Shift') ? 3 : 1);
    eye.addScaledVector(v, speed * dt);
    if (walk) eye.z = 1.7; else eye.z = Math.max(0.6, eye.z);
    // out of whatever it has gone into, the shortest way
    const m = walk ? 0.5 : 1.2;
    for (const s of solids) {
      if (eye.x <= s[0] - m || eye.x >= s[3] + m || eye.y <= s[1] - m || eye.y >= s[4] + m || eye.z <= s[2] - m || eye.z >= s[5] + m) continue;
      const out = [eye.x - (s[0] - m), (s[3] + m) - eye.x, eye.y - (s[1] - m), (s[4] + m) - eye.y];
      if (!walk) out.push(s[2] - m > 0.6 ? eye.z - (s[2] - m) : Infinity, (s[5] + m) - eye.z);
      const least = out.indexOf(Math.min(...out));
      if (least === 0) eye.x = s[0] - m;
      else if (least === 1) eye.x = s[3] + m;
      else if (least === 2) eye.y = s[1] - m;
      else if (least === 3) eye.y = s[4] + m;
      else if (least === 4) eye.z = s[2] - m;
      else eye.z = s[5] + m;
    }
  };

  const wrap = (c) => ((c + 420) % 840 + 840) % 840 - 420;
  const texel = new THREE.Matrix4().set(0.5, 0, 0, 0.5, 0, 0.5, 0, 0.5, 0, 0, 0.5, 0.5, 0, 0, 0, 1);
  let last = performance.now(), running = true, shown = 0, rate = 0, since = last;
  const frame = (now) => {
    if (!running) return;
    const dt = Math.min(0.1, (now - last) / 1000);
    last = now;
    if (!state.held) state.time += dt;
    shared.uTime.value = state.time;
    for (const m of movers) {
      const u = state.time / film.seconds, p = [0, 1, 2].map((k) => m.from[k] + (m.to[k] - m.from[k]) * u);
      const axis = Math.abs(m.to[0] - m.from[0]) > Math.abs(m.to[1] - m.from[1]) ? 0 : 1;
      if (!state.held) p[axis] = wrap(p[axis]);
      m.g.position.set(p[0], p[1], p[2]);
    }
    if (state.mode === 'ride') {
      const t = state.held ? state.time : state.time % film.seconds;
      const e = along(film.eyes, t), a = along(film.aims, t);
      camera.position.set(e[0], e[1], e[2]);
      camera.lookAt(a[0], a[1], a[2]);
    } else {
      step(dt);
      camera.position.copy(eye);
      camera.lookAt(eye.clone().add(facing()));
    }
    camera.updateMatrixWorld();
    // the street's mirror: the scene from under the ground, looking up
    const d = new THREE.Vector3();
    camera.getWorldDirection(d);
    below.fov = camera.fov; below.aspect = camera.aspect; below.near = camera.near; below.far = camera.far;
    below.updateProjectionMatrix();
    below.position.set(camera.position.x, camera.position.y, -camera.position.z);
    below.lookAt(camera.position.x + d.x, camera.position.y + d.y, -camera.position.z - d.z);
    below.updateMatrixWorld();
    mirror.uMirrorMatrix.value.copy(texel).multiply(below.projectionMatrix).multiply(below.matrixWorldInverse);
    for (const g of grounds) g.visible = false;
    renderer.setRenderTarget(target);
    renderer.render(scene, below);
    renderer.setRenderTarget(null);
    for (const g of grounds) g.visible = true;
    composer.render();
    shown += 1;
    if (now - since > 500) { rate = shown * 1000 / (now - since); shown = 0; since = now; }
    if (options.onFrame) options.onFrame({ time: state.time, rate, mode: state.mode, eye: camera.position });
    requestAnimationFrame(frame);
  };
  requestAnimationFrame(frame);

  return {
    report: api.reported,
    mode: (m) => { if (m === 'ride') { state.mode = 'ride'; if (options.onMode) options.onMode(m); } else leave(m); },
    stop() {
      running = false;
      gone.abort();
      if (document.pointerLockElement === el) document.exitPointerLock();
      scene.traverse((o) => { if (o.geometry) o.geometry.dispose(); if (o.material) o.material.dispose(); });
      target.dispose();
      composer.dispose();
      renderer.dispose();
      renderer.forceContextLoss();
      el.remove();
    },
  };
}
