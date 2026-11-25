// The board's water. A module of the board player (../board.js says
// what a module is). One call, kept by its line:
//   water(nums('x z y ...', 3), ints('a b c ...', 3), nums('fx fz', 2)[0], '#3f6f8f')
//     corners    the surface's corners: x east, z south, y up, in tiles
//     triangles  three corners each, as indexes into the corners,
//                counted from 1 as X_eTaL counts (the first corner is
//                1), as the terrain call's are. Either winding: a
//                triangle is turned to face up. An index that is 0, too
//                large or not whole is an error naming the triangle.
//     flow       the way the water moves and how fast, tiles a second,
//                along x and along z; 0 0 is still water
//     color      the water's own color, as the light leaves it
// Several water lines may be in one scene. A line whose text is
// unchanged is kept; a changed line is one surface disposed and one
// made.
//
// What is drawn. A standard material (so the board's lights, shadows,
// fog and tone mapping apply to it as to anything else), changed in
// three places:
// - ripples: its normal is tilted by two layers of one small picture of
//   waves (made here, once, from a fixed seed), stretched along the flow
//   and scrolled by flow times the board's clock. Nothing is kept from
//   frame to frame, so a held clock (?t=) gives the same picture every
//   time and a scene shown again does not restart anything. Still water
//   still shimmers: a drift of about 0.03 tiles a second.
// - sky: the color the sky has in the mirrored direction (from
//   scene.background at the horizon to ctx.lights.sky's color overhead)
//   mixed in by a fresnel term: little looking down, most at a glance.
//   No second picture of the scene is drawn, so nothing but the sky is
//   mirrored.
// - foam: where the water meets anything else. The scene is drawn once
//   more each frame, from straight above, without perspective, into a
//   small map (about 24 dots a tile over the waters' box, one map for
//   all the water lines): for each dot, how far under the water's
//   surface the highest thing under it is, and 0 where the surface is
//   inside something solid (a bank, a beam, a leg). Foam is drawn where
//   that depth is small, drifting cells of it, and trailing downstream
//   of whatever stands in the flow. Looking from above means the foam
//   does not depend on the camera at all: it is the same with and
//   without perspective, on every side of a thing, and behind it. The
//   same map makes shallow water clearer and lighter than deep water.
//   What it costs: the scene's corners drawn a second time (the dots
//   are few), and it is wrong for water under water, as seen from
//   above (a fall's sheet over a pool: the higher surface counts), and
//   for a shape that is not closed (a lone sheet seen from its back
//   reads as solid).
import * as THREE from 'three';

const FIRST = 1;        // the first corner's index in a triangle
const DOTS = 24;       // the foam map's dots a tile
const WIDEST = 2048;    // and its longest side, in dots
const MARGIN = 0.75;    // tiles of the scene round the water that the map holds
const BELOW = 8;        // how far under the lowest water the map looks, in tiles
const FAR = 100;        // the map's depth where there is nothing under the water
const WAVES = 256;      // the side of the picture of waves
const A = 1 / 3.1, B = 1 / 1.9, TURN = 0.9, F = 1.3, U = 0.23; // the two layers' and the foam's tiles to a picture

// a small generator of numbers that are the same every time
const random = (seed) => () => {
  seed = (seed + 0x6d2b79f5) | 0;
  let t = Math.imul(seed ^ (seed >>> 15), 1 | seed);
  t = (t + Math.imul(t ^ (t >>> 7), 61 | t)) ^ t;
  return ((t ^ (t >>> 14)) >>> 0) / 4294967296;
};

// The picture of waves, repeating at its edges: red and green are a
// surface's slope along and across (many waves of whole numbers of
// crests a side, so it repeats), blue is foam's cells (how far the
// nearest of some scattered points is), alpha a slower unevenness.
function waves() {
  const n = WAVES, r = random(7), data = new Uint8Array(n * n * 4);
  const list = [], slow = [], points = [];
  for (let i = 0; i < 28; i++) {
    const turn = r() * Math.PI * 2, k = 3 + r() * 11;
    const kx = Math.round(Math.cos(turn) * k), ky = Math.round(Math.sin(turn) * k);
    list.push([kx, ky, r() * Math.PI * 2, 1 / Math.pow(Math.hypot(kx, ky) || 1, 1.7)]);
  }
  for (let i = 0; i < 6; i++) slow.push([Math.round(r() * 6 - 3), Math.round(r() * 6 - 3), r() * Math.PI * 2]);
  for (let i = 0; i < 44; i++) points.push([r(), r()]);
  const sx = new Float32Array(n * n), sy = new Float32Array(n * n);
  let most = 0; // the slopes' spread: red and green hold 2.6 of it
  for (let y = 0; y < n; y++) {
    for (let x = 0; x < n; x++) {
      const u = x / n * Math.PI * 2, v = y / n * Math.PI * 2;
      let gx = 0, gy = 0;
      for (const [kx, ky, phase, tall] of list) {
        const s = -Math.sin(kx * u + ky * v + phase) * tall;
        gx += s * kx;
        gy += s * ky;
      }
      sx[y * n + x] = gx;
      sy[y * n + x] = gy;
      most += gx * gx + gy * gy;
    }
  }
  most = 2.6 * Math.sqrt(most / (2 * n * n));
  for (let y = 0; y < n; y++) {
    for (let x = 0; x < n; x++) {
      const i = y * n + x, u = x / n, v = y / n;
      let near = 1;
      for (const [px, py] of points) {
        let dx = Math.abs(u - px), dy = Math.abs(v - py);
        if (dx > 0.5) dx = 1 - dx;
        if (dy > 0.5) dy = 1 - dy;
        near = Math.min(near, dx * dx + dy * dy);
      }
      let uneven = 0;
      for (const [kx, ky, phase] of slow) uneven += Math.cos((kx * u + ky * v) * Math.PI * 2 + phase);
      data[i * 4] = Math.round(127.5 + 127.5 * Math.max(-1, Math.min(1, sx[i] / most)));
      data[i * 4 + 1] = Math.round(127.5 + 127.5 * Math.max(-1, Math.min(1, sy[i] / most)));
      data[i * 4 + 2] = Math.round(255 * Math.min(1, Math.sqrt(near) * 7.5));
      data[i * 4 + 3] = Math.round(255 * Math.min(1, Math.max(0, 0.5 + uneven / 7)));
    }
  }
  const t = new THREE.DataTexture(data, n, n, THREE.RGBAFormat);
  t.wrapS = t.wrapT = THREE.RepeatWrapping;
  t.magFilter = THREE.LinearFilter;
  t.minFilter = THREE.LinearMipmapLinearFilter;
  t.generateMipmaps = true;
  t.anisotropy = 4;
  t.needsUpdate = true;
  return t;
}

// what the standard material's shader gains
const HEAD_VERTEX = 'varying vec3 vWaterAt;\n';
const AT_VERTEX = '\nvWaterAt = (modelMatrix * vec4(transformed, 1.0)).xyz;\n';
const HEAD_FRAGMENT = `
uniform sampler2D waterWaves;
uniform sampler2D waterShore;
uniform mat4 waterShoreAt;
uniform vec4 waterFlow;    // the flow's direction (x, z), its speed, the stretch along it
uniform vec4 waterScroll;  // how far the two layers have gone
uniform vec4 waterFoamScroll; // how far the foam's cells have gone, and its unevenness
uniform vec3 waterSkyTop;
uniform vec3 waterSkyLow;
varying vec3 vWaterAt;
float waterFoam;
float waterNear(vec2 at) {
  return 1.0 - clamp(texture2D(waterShore, at).r / 0.12, 0.0, 1.0);
}
`;
// the color: clearer and lighter where it is shallow, white where it foams
const COLOR_FRAGMENT = `
vec2 waterDir = waterFlow.xy, waterSide = vec2(-waterFlow.y, waterFlow.x);
vec2 waterQ = vec2(dot(vWaterAt.xz, waterDir), dot(vWaterAt.xz, waterSide));
vec2 waterShoreUv = (waterShoreAt * vec4(vWaterAt, 1.0)).xy * 0.5 + 0.5;
vec2 waterAlong = (waterShoreAt * vec4(waterDir.x, 0.0, waterDir.y, 0.0)).xy * 0.5;
vec2 waterAcross = (waterShoreAt * vec4(waterSide.x, 0.0, waterSide.y, 0.0)).xy * 0.5;
float waterDeep = texture2D(waterShore, waterShoreUv).r;
float waterEdge = waterNear(waterShoreUv) * 0.8 + 0.4 * (
  waterNear(waterShoreUv + waterAlong * 0.16) + waterNear(waterShoreUv - waterAlong * 0.16) +
  waterNear(waterShoreUv + waterAcross * 0.16) + waterNear(waterShoreUv - waterAcross * 0.16));
float waterWake = max(waterNear(waterShoreUv - waterAlong * 0.3), 0.6 * waterNear(waterShoreUv - waterAlong * 0.6)) * min(waterFlow.z, 1.0);
vec4 waterCells = texture2D(waterWaves, waterQ * ${F.toFixed(4)} - waterFoamScroll.xy);
float waterUneven = texture2D(waterWaves, waterQ * ${U.toFixed(4)} - waterFoamScroll.zw).a;
float waterMuch = clamp(max(waterEdge, waterWake * 0.75), 0.0, 1.0);
float waterGate = 0.14 + 0.86 * mix(waterCells.b, waterUneven, 0.35);
waterFoam = smoothstep(waterGate - 0.08, waterGate + 0.08, min(waterMuch * 1.2, 0.85));
float waterDepth = smoothstep(0.0, 1.0, waterDeep);
diffuseColor.rgb *= mix(1.3, 0.8, waterDepth);
diffuseColor.a *= mix(0.62, 0.94, waterDepth);
diffuseColor = mix(diffuseColor, vec4(0.94, 0.96, 0.97, 0.96), waterFoam);
`;
// the normal: the surface's own, tilted by the two layers of waves
const NORMAL_FRAGMENT = `
vec2 waterA = vec2(waterQ.x / waterFlow.w, waterQ.y) * ${A.toFixed(4)} - waterScroll.xy;
mat2 waterTurn = mat2(${Math.cos(TURN).toFixed(4)}, ${Math.sin(TURN).toFixed(4)}, ${(-Math.sin(TURN)).toFixed(4)}, ${Math.cos(TURN).toFixed(4)});
vec2 waterB = (waterTurn * waterQ) * ${B.toFixed(4)} - waterScroll.zw;
vec2 waterSa = texture2D(waterWaves, waterA).rg * 2.0 - 1.0;
vec2 waterSb = texture2D(waterWaves, waterB).rg * 2.0 - 1.0;
vec2 waterSq = vec2(waterSa.x / waterFlow.w, waterSa.y) * 0.26 + (waterSb * waterTurn) * 0.2;
vec2 waterSw = (waterDir * waterSq.x + waterSide * waterSq.y) * (1.0 - 0.7 * waterFoam);
vec3 waterUp = inverseTransformDirection(normal, viewMatrix);
vec3 waterN = normalize(waterUp - vec3(waterSw.x, 0.0, waterSw.y));
normal = normalize((viewMatrix * vec4(waterN, 0.0)).xyz);
`;
const ROUGH_FRAGMENT = '\nroughnessFactor = mix(roughnessFactor, 0.9, waterFoam);\n';
// the sky, mirrored: its color in the mirrored direction, by a fresnel term
const SKY_FRAGMENT = `
vec3 waterSee = normalize(vViewPosition);
if (isOrthographic) waterSee = vec3(0.0, 0.0, 1.0);
float waterFacing = clamp(dot(normal, waterSee), 0.0, 1.0);
float waterMirror = (0.02 + 0.98 * pow(1.0 - waterFacing, 3.0)) * (1.0 - waterFoam);
vec3 waterBack = inverseTransformDirection(reflect(-waterSee, normal), viewMatrix);
vec3 waterSky = mix(waterSkyLow, waterSkyTop, 0.3 + 0.7 * sqrt(clamp(waterBack.y, 0.0, 1.0)));
outgoingLight = mix(outgoingLight, waterSky, waterMirror);
diffuseColor.a = mix(diffuseColor.a, 1.0, waterMirror);
`;

// the foam map's two materials: the water's own height, and the scene under it
const heightMaterial = () => new THREE.ShaderMaterial({
  uniforms: { shift: { value: new THREE.Vector2() } },
  vertexShader: `
    uniform vec2 shift;
    varying float vHigh;
    void main() {
      vec4 at = modelMatrix * vec4(position, 1.0);
      vHigh = at.y;
      gl_Position = projectionMatrix * viewMatrix * at;
      gl_Position.xy += shift;
    }`,
  fragmentShader: 'varying float vHigh;\nvoid main() { gl_FragColor = vec4(vHigh, 0.0, 0.0, 1.0); }',
  side: THREE.DoubleSide, depthTest: false, depthWrite: false,
});
const underMaterial = () => new THREE.ShaderMaterial({
  uniforms: { heights: { value: null }, texel: { value: new THREE.Vector2(1, 1) }, top: { value: new THREE.Vector3() } },
  vertexShader: `
    #include <common>
    #include <batching_pars_vertex>
    #include <morphtarget_pars_vertex>
    #include <skinning_pars_vertex>
    void main() {
      #include <batching_vertex>
      #include <skinbase_vertex>
      #include <begin_vertex>
      #include <morphinstance_vertex>
      #include <morphtarget_vertex>
      #include <skinning_vertex>
      #include <project_vertex>
    }`,
  // top: the map camera's height, its near and its far; a dot is what the
  // highest thing under the water is under it by, 0 seen from inside
  fragmentShader: `
    uniform sampler2D heights;
    uniform vec2 texel;
    uniform vec3 top;
    void main() {
      float high = top.x - (top.y + gl_FragCoord.z * (top.z - top.y));
      float surface = texture2D(heights, gl_FragCoord.xy * texel).r;
      if (high > surface + 0.0005) discard;
      gl_FragColor = vec4(gl_FrontFacing ? surface - high : 0.0, 0.0, 0.0, 1.0);
    }`,
  side: THREE.DoubleSide,
});

export default function water(ctx) {
  const { scene, renderer } = ctx;
  const waters = new Set();
  const nothing = new THREE.DataTexture(new Float32Array([FAR, 0, 0, 1]), 1, 1, THREE.RGBAFormat, THREE.FloatType);
  nothing.needsUpdate = true;
  const eye = new THREE.OrthographicCamera(-1, 1, 1, -1, 0.01, 10);
  const shoreAt = new THREE.Matrix4();
  const was = new THREE.Color(), clear = new THREE.Color(), hidden = [];
  let picture = null, heights = null, shore = null, under = null, stale = true, triangles = 0;

  const target = (wide, high, depth) => new THREE.WebGLRenderTarget(wide, high, {
    type: THREE.HalfFloatType, format: THREE.RGBAFormat, depthBuffer: depth,
    minFilter: THREE.LinearFilter, magFilter: THREE.LinearFilter, generateMipmaps: false,
  });
  const drop = () => {
    for (const t of [heights, shore]) if (t) t.dispose();
    heights = shore = null;
  };

  // with the renderer drawing into a map of ours, then as it was
  const into = (to, draw) => {
    const before = { to: renderer.getRenderTarget(), alpha: renderer.getClearAlpha(), auto: renderer.autoClear, shadows: renderer.shadowMap.autoUpdate, xr: renderer.xr.enabled };
    renderer.getClearColor(was);
    renderer.xr.enabled = false;
    renderer.shadowMap.autoUpdate = false;
    renderer.autoClear = false;
    renderer.setRenderTarget(to);
    try {
      draw();
    } finally {
      renderer.setRenderTarget(before.to);
      renderer.setClearColor(was, before.alpha);
      renderer.autoClear = before.auto;
      renderer.shadowMap.autoUpdate = before.shadows;
      renderer.xr.enabled = before.xr;
    }
  };

  // The map's place and size, for the waters there are now, and the
  // height of their surfaces in it: each surface drawn from above, and
  // first drawn again a margin to every side, so the banks beside the
  // water have a surface to be under.
  const lay = () => {
    stale = false;
    const box = new THREE.Box3();
    for (const w of waters) box.union(w.box);
    if (box.isEmpty()) { drop(); return; }
    const wide = box.max.x - box.min.x + MARGIN * 2, long = box.max.z - box.min.z + MARGIN * 2;
    const dots = Math.min(DOTS, WIDEST / Math.max(wide, long));
    const w = Math.max(4, Math.ceil(wide * dots)), h = Math.max(4, Math.ceil(long * dots));
    if (!shore || shore.width !== w || shore.height !== h) {
      drop();
      heights = target(w, h, false);
      shore = target(w, h, true);
    }
    eye.left = -wide / 2; eye.right = wide / 2; eye.top = long / 2; eye.bottom = -long / 2;
    eye.near = 0.5;
    eye.far = 1 + (box.max.y - box.min.y) + BELOW;
    eye.position.set((box.min.x + box.max.x) / 2, box.max.y + 1, (box.min.z + box.max.z) / 2);
    eye.up.set(0, 0, -1);
    eye.lookAt(eye.position.x, box.min.y - BELOW, eye.position.z);
    eye.updateProjectionMatrix();
    eye.updateMatrixWorld(true);
    shoreAt.multiplyMatrices(eye.projectionMatrix, eye.matrixWorldInverse);
    if (!under) under = { scene: underMaterial(), high: heightMaterial() };
    under.scene.uniforms.heights.value = heights.texture;
    under.scene.uniforms.texel.value.set(1 / w, 1 / h);
    under.scene.uniforms.top.value.set(eye.position.y, eye.near, eye.far);
    const all = new THREE.Scene();
    for (const one of waters) all.add(new THREE.Mesh(one.surface.geometry, under.high));
    into(heights, () => {
      // red is a height: cleared to far under everything
      renderer.setClearColor(clear.setRGB(-1e4, 0, 0), 1);
      renderer.clear();
      const sx = MARGIN * 2 / wide, sz = MARGIN * 2 / long;
      for (const [x, z] of [[1, 0], [-1, 0], [0, 1], [0, -1], [1, 1], [1, -1], [-1, 1], [-1, -1], [0.5, 0], [-0.5, 0], [0, 0.5], [0, -0.5], [0, 0]]) {
        under.high.uniforms.shift.value.set(x * sx, z * sz);
        renderer.render(all, eye);
      }
    });
    for (const one of waters) {
      one.uniforms.waterShore.value = shore.texture;
      one.uniforms.waterShoreAt.value = shoreAt;
    }
  };

  // the scene under the water, from above: what is solid, not the water
  // itself, nor anything see-through, nor points, lines and sprites
  const solid = (o) => {
    if (!o.isMesh) return !(o.isPoints || o.isLine || o.isSprite);
    const m = Array.isArray(o.material) ? o.material[0] : o.material;
    return !o.userData.water && m && !m.transparent && m.depthWrite !== false && m.side !== THREE.BackSide;
  };
  const look = () => {
    hidden.length = 0;
    scene.traverseVisible((o) => { if (!solid(o)) hidden.push(o); });
    for (const o of hidden) o.visible = false;
    const back = scene.background, over = scene.overrideMaterial;
    scene.background = null;
    scene.overrideMaterial = under.scene;
    try {
      into(shore, () => {
        renderer.setClearColor(clear.setRGB(FAR, 0, 0), 1);
        renderer.clear();
        renderer.render(scene, eye);
      });
    } finally {
      scene.background = back;
      scene.overrideMaterial = over;
      for (const o of hidden) o.visible = true;
      hidden.length = 0;
    }
  };

  const make = (line, corners, tris, flow, color) => {
    if (!picture) picture = waves();
    const at = new Float32Array(corners.length * 3);
    corners.forEach((c, i) => { at[i * 3] = c[0]; at[i * 3 + 1] = c[2]; at[i * 3 + 2] = c[1]; });
    // every triangle facing up, whichever way round it was written
    const index = [];
    tris.forEach((t, n) => {
      const last = corners.length + FIRST - 1;
      if (t.length !== 3 || !t.every((i) => Number.isInteger(i) && i >= FIRST && i <= last)) {
        throw new Error(`water: triangle ${n + FIRST} (${t.join(' ')}) names a corner that is not there: corners are counted from ${FIRST}, and this surface has ${FIRST} to ${last}`);
      }
      const [a, b, c] = t.map((i) => i - FIRST);
      const ux = at[b * 3] - at[a * 3], uz = at[b * 3 + 2] - at[a * 3 + 2], vx = at[c * 3] - at[a * 3], vz = at[c * 3 + 2] - at[a * 3 + 2];
      if (uz * vx - ux * vz < 0) index.push(a, c, b); else index.push(a, b, c);
    });
    const geometry = new THREE.BufferGeometry();
    geometry.setAttribute('position', new THREE.BufferAttribute(at, 3));
    geometry.setIndex(index);
    geometry.computeVertexNormals();
    geometry.computeBoundingBox();
    geometry.computeBoundingSphere();
    const fx = Number(flow && flow[0]) || 0, fz = Number(flow && flow[1]) || 0, speed = Math.hypot(fx, fz);
    const dir = speed > 0 ? [fx / speed, fz / speed] : [1, 0];
    const uniforms = {
      waterWaves: { value: picture },
      waterShore: { value: nothing },
      waterShoreAt: { value: shoreAt },
      waterFlow: { value: new THREE.Vector4(dir[0], dir[1], speed, 1 + Math.min(speed, 1.5) * 1.2) },
      waterScroll: { value: new THREE.Vector4() },
      waterFoamScroll: { value: new THREE.Vector4() },
      waterSkyTop: { value: new THREE.Color(1, 1, 1) },
      waterSkyLow: { value: new THREE.Color(0.5, 0.5, 0.5) },
    };
    const material = new THREE.MeshStandardMaterial({ color, roughness: 0.16, metalness: 0, transparent: true, depthWrite: false });
    material.onBeforeCompile = (shader) => {
      Object.assign(shader.uniforms, uniforms);
      shader.vertexShader = HEAD_VERTEX + shader.vertexShader.replace('#include <project_vertex>', '#include <project_vertex>' + AT_VERTEX);
      shader.fragmentShader = HEAD_FRAGMENT + shader.fragmentShader
        .replace('#include <color_fragment>', '#include <color_fragment>' + COLOR_FRAGMENT)
        .replace('#include <roughnessmap_fragment>', '#include <roughnessmap_fragment>' + ROUGH_FRAGMENT)
        .replace('#include <normal_fragment_maps>', '#include <normal_fragment_maps>' + NORMAL_FRAGMENT)
        .replace('#include <opaque_fragment>', SKY_FRAGMENT + '#include <opaque_fragment>');
    };
    material.customProgramCacheKey = () => 'board-water';
    const mesh = new THREE.Mesh(geometry, material);
    mesh.name = 'water';
    mesh.userData.water = true;
    mesh.receiveShadow = true;
    scene.add(mesh);
    const one = {
      surface: mesh, uniforms, speed, box: geometry.boundingBox, triangles: index.length / 3,
      dispose() {
        waters.delete(one);
        scene.remove(mesh);
        geometry.dispose();
        material.dispose();
        triangles -= one.triangles;
        stale = true;
      },
    };
    waters.add(one);
    triangles += one.triangles;
    stale = true;
    return one;
  };

  // a part of a number, never negative: how far into its picture a layer is
  const part = (x) => x - Math.floor(x);
  const cos = Math.cos(TURN), sin = Math.sin(TURN);
  const sky = new THREE.Color(), low = new THREE.Color();

  return {
    kept: { water: make },
    frame(time) {
      if (!waters.size) { if (shore) drop(); return; }
      const light = ctx.lights && ctx.lights.sky, back = scene.background;
      if (light && light.color) sky.copy(light.color); else sky.setRGB(1, 1, 1);
      if (back && back.isColor) low.copy(back);
      else if (scene.fog && scene.fog.color) low.copy(scene.fog.color);
      else if (light && light.groundColor) low.copy(light.groundColor);
      else low.copy(sky).multiplyScalar(0.6);
      for (const w of waters) {
        // each layer has gone its own speed along the flow, and a slow drift
        const stretch = w.uniforms.waterFlow.value.w;
        const ax = (w.speed + 0.03) * time, ay = 0.02 * time, bx = (w.speed * 0.65 - 0.025) * time, by = 0.03 * time;
        w.uniforms.waterScroll.value.set(part(ax / stretch * A), part(ay * A), part((cos * bx - sin * by) * B), part((sin * bx + cos * by) * B));
        w.uniforms.waterFoamScroll.value.set(part(w.speed * time * F), part(0.011 * time * F), part(w.speed * time * U), part(0.011 * time * U));
        w.uniforms.waterSkyTop.value.copy(sky);
        w.uniforms.waterSkyLow.value.copy(low);
      }
      if (stale) lay();
      if (shore) look();
    },
    count() {
      return { water: waters.size, waterTriangles: triangles };
    },
    stop() {
      for (const w of [...waters]) w.dispose();
      drop();
      if (picture) picture.dispose();
      if (under) { under.scene.dispose(); under.high.dispose(); }
      nothing.dispose();
    },
  };
}
