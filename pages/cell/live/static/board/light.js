// The board's light. A module of the board player (../board.js says
// what a module is). Three calls, each kept by its line:
//   sun(nums('dx dy dz', 3)[0], color, strength)
//                               the direction toward the sun (x east, y
//                               up, z south; any length), its color and
//                               how strong it is; it casts shadows
//   sky(top, horizon, strength) the sky's light: top from above, horizon
//                               from below; seen behind the scene by a
//                               camera with perspective (view's
//                               background stays behind a board's own)
//   fog(color, density)         fog with distance: what is d tiles away
//                               keeps exp(-(density d)^2) of itself
// A line gone puts that thing back to the board's default, which is
//   sun(nums('-0.7 0.9 0.35', 3)[0], '#fff4e6', 2.6)
//   sky('#cfdcf0', '#7d7668', 1.1)
//   no fog
// and of the same call twice, the later line counts. Always on:
// shadows from the sun (one map, fitted to what the camera sees and
// moved a whole texel at a time, so they hold still), tone mapping
// (AgX, exposure EXPOSURE), and ambient occlusion (off with the
// board's option occlusion: false; then the picture is drawn straight
// to the screen).
//
// Distance for fog is from the eye. A camera without perspective has
// no eye, so there it is from the front of the picture: the plane,
// facing the camera, FRONT in front of its near plane (board.js puts
// that plane through the ground at the bottom middle of the view).
//
// For the other modules: scene.fog is a FogExp2 or null;
// renderer.toneMapping is AgX; renderer.shadowMap is on; ctx.lights
// holds sky (a HemisphereLight) and sun (a DirectionalLight). With
// occlusion the scene is drawn into a target (linear, not tone mapped;
// its alpha is what covers the background) and a last pass tone maps
// it, so a material needs only Three's own chunks, as board.js says.
import * as THREE from 'three';

export const FRONT = 200;
export const EXPOSURE = 1.25;
const SUN = { direction: [-0.7, 0.9, 0.35], color: '#fff4e6', strength: 2.6 };
const SKY = { top: '#cfdcf0', horizon: '#7d7668', strength: 1.1 };
const MAP = 2048;       // the shadow map's side, texels
const REACH = 30;       // how far from an eye with perspective shadows fall
const RADIUS = 0.55;    // how far occlusion looks, tiles
const DARK = 0.75;      // how dark a full corner gets

// fog under a camera without perspective: from the front of the
// picture, not from the camera, which stands far off (its near plane
// is read back from the projection: no material needs to know)
const DEPTH = 'vFogDepth = - mvPosition.z;';
if (THREE.ShaderChunk.fog_vertex.includes(DEPTH) && !THREE.ShaderChunk.fog_vertex.includes('projectionMatrix')) {
  THREE.ShaderChunk.fog_vertex = THREE.ShaderChunk.fog_vertex.replace(DEPTH, `${DEPTH}
	if ( projectionMatrix[ 3 ][ 3 ] > 0.5 ) vFogDepth = max( 0.0, vFogDepth - ( projectionMatrix[ 3 ][ 2 ] + 1.0 ) / projectionMatrix[ 2 ][ 2 ] - ${FRONT.toFixed(1)} );`);
}

const WHOLE = `
varying vec2 vUv;
void main() {
  vUv = position.xy * 0.5 + 0.5;
  gl_Position = vec4(position.xy, 0.0, 1.0);
}`;

// how much of the space round a point is filled, from the depths drawn:
// pairs of places opposite each other, so a flat face (one of each pair
// under it) counts as open and only a corner darkens
const OCCLUSION = `
uniform sampler2D depths;
uniform mat4 lens, back;
uniform float radius;
varying vec2 vUv;
vec3 place(vec2 uv, float d) {
  vec4 p = back * vec4(uv * 2.0 - 1.0, d * 2.0 - 1.0, 1.0);
  return p.xyz / p.w;
}
// 1 hidden behind what is drawn, 0 in the open, -1 too far behind to say
float hidden(vec3 q, float slack) {
  vec4 c = lens * vec4(q, 1.0);
  vec2 uv = c.xy / c.w * 0.5 + 0.5;
  float gap = place(uv, texture2D(depths, uv).x).z - q.z;
  return gap > radius * 3.0 ? -1.0 : step(slack, gap);
}
void main() {
  float d = texture2D(depths, vUv).x;
  if (d >= 1.0) { gl_FragColor = vec4(1.0); return; }
  vec3 p = place(vUv, d);
  // a face seen along itself is deeper a pixel on: that much is not a corner
  float slack = radius * 0.05 + 2.0 * (abs(dFdx(p.z)) + abs(dFdy(p.z)));
  // sixteen turns, a square of four by four pixels: the last pass takes their mean
  vec2 cell = mod(floor(gl_FragCoord.xy), 4.0);
  float a = 3.1416 * (cell.x * 5.0 + cell.y * 4.0 + mod(cell.x * cell.y, 4.0)) / 16.0;
  mat2 turn = mat2(cos(a), sin(a), -sin(a), cos(a));
  vec3 round[6];
  round[0] = vec3(0.80, 0.10, 0.25);
  round[1] = vec3(-0.35, 0.65, 0.30);
  round[2] = vec3(-0.20, -0.75, 0.45);
  round[3] = vec3(0.32, -0.28, 0.52);
  round[4] = vec3(0.05, 0.18, 0.45);
  round[5] = vec3(-0.21, -0.05, 0.25);
  float sum = 0.0;
  for (int i = 0; i < 6; i++) {
    vec3 k = round[i] * radius;
    k.xy = turn * k.xy;
    float f = hidden(p + k, slack), b = hidden(p - k, slack);
    if (f > 0.5 && b > 0.5) sum += 1.0;
  }
  gl_FragColor = vec4(vec3(1.0 - min(1.0, sum / 3.0)), 1.0);
}`;

// the last pass: the scene, darkened in its corners, tone mapped, over
// the view's background (which is not tone mapped: it is the color asked for)
const LAST = `
uniform sampler2D drawn, corners;
uniform vec2 texel;
uniform vec3 behind;
uniform float dark;
varying vec2 vUv;
void main() {
  vec4 c = texture2D(drawn, vUv);
  float open = 0.25 * (texture2D(corners, vUv + texel * vec2(-0.5, -0.5)).x + texture2D(corners, vUv + texel * vec2(1.5, -0.5)).x
    + texture2D(corners, vUv + texel * vec2(-0.5, 1.5)).x + texture2D(corners, vUv + texel * vec2(1.5, 1.5)).x);
  gl_FragColor = vec4(c.a > 0.0 ? c.rgb / c.a * (1.0 - dark * (1.0 - open)) : vec3(0.0), 1.0);
  #include <tonemapping_fragment>
  #include <colorspace_fragment>
  gl_FragColor = vec4(mix(behind, gl_FragColor.rgb, clamp(c.a, 0.0, 1.0)), 1.0);
}`;

// the sky behind everything, for a camera with perspective
const DOME_AT = `
varying vec3 way;
void main() {
  vec4 p = inverse(projectionMatrix) * vec4(position.xy, 1.0, 1.0);
  way = transpose(mat3(viewMatrix)) * (p.xyz / p.w);
  gl_Position = vec4(position.xy, 1.0, 1.0);
}`;
const DOME = `
uniform vec3 top, horizon, low;
varying vec3 way;
void main() {
  float y = normalize(way).y;
  vec3 c = mix(horizon, top, pow(clamp(y, 0.0, 1.0), 0.6));
  c = mix(low, c, smoothstep(-0.02, 0.12, y));
  gl_FragColor = vec4(c, 1.0);
  #include <tonemapping_fragment>
  #include <colorspace_fragment>
}`;

export default function light(ctx) {
  const { scene, renderer, options, look } = ctx;
  const { sky, sun } = ctx.lights;
  const post = options.occlusion !== false;
  const lines = { sun: [], sky: [], fog: [] };

  renderer.toneMapping = THREE.AgXToneMapping;
  renderer.toneMappingExposure = EXPOSURE;
  renderer.shadowMap.enabled = true;
  renderer.shadowMap.type = THREE.PCFShadowMap;
  sun.castShadow = true;
  sun.shadow.mapSize.set(MAP, MAP);
  sun.shadow.bias = -0.0004;
  sun.shadow.radius = 1.5;
  scene.add(sun.target);

  const whole = new THREE.BufferGeometry();
  whole.setAttribute('position', new THREE.Float32BufferAttribute([-1, -1, 0, 3, -1, 0, -1, 3, 0], 3));
  const still = new THREE.Camera();
  const pass = (fragmentShader, uniforms, more = {}) => {
    const mesh = new THREE.Mesh(whole, new THREE.ShaderMaterial({ vertexShader: WHOLE, fragmentShader, uniforms, depthTest: false, depthWrite: false, ...more }));
    mesh.frustumCulled = false;
    return mesh;
  };

  const dome = new THREE.Mesh(whole, new THREE.ShaderMaterial({
    vertexShader: DOME_AT, fragmentShader: DOME, depthTest: false, depthWrite: false, fog: false,
    uniforms: { top: { value: new THREE.Color() }, horizon: { value: new THREE.Color() }, low: { value: new THREE.Color() } },
  }));
  dome.name = 'sky';
  dome.frustumCulled = false;
  dome.renderOrder = -1000;
  dome.visible = false;
  scene.add(dome);

  // what the lines in force say (the last of each), or the default
  const apply = () => {
    const s = lines.sun.at(-1) || SUN, k = lines.sky.at(-1) || SKY, f = lines.fog.at(-1);
    sun.color.set(s.color);
    sun.intensity = s.strength;
    sun.userData.toward = s.direction;
    sky.color.set(k.top);
    sky.groundColor.set(k.horizon);
    sky.intensity = k.strength;
    if (f) {
      if (!scene.fog) scene.fog = new THREE.FogExp2(f.color, f.density);
      scene.fog.color.set(f.color);
      scene.fog.density = f.density;
    } else {
      scene.fog = null;
    }
    dome.material.uniforms.top.value.set(k.top);
    dome.material.uniforms.horizon.value.set(k.horizon);
    dome.material.uniforms.low.value.set(f ? f.color : k.horizon);
    dome.userData.given = lines.sky.length > 0;
  };
  const line = (list, made) => {
    list.push(made);
    apply();
    return { dispose() { list.splice(list.indexOf(made), 1); apply(); } };
  };
  const number = (x, otherwise) => (Number.isFinite(x) ? x : otherwise);
  const kept = {
    sun(text, direction, color, strength) {
      const d = (direction || []).map((x) => number(x, 0));
      const usable = d.length >= 3 && Math.hypot(d[0], d[1], d[2]) > 0;
      return line(lines.sun, { direction: usable ? d : SUN.direction, color, strength: Math.max(0, number(strength, SUN.strength)) });
    },
    sky(text, top, horizon, strength) {
      return line(lines.sky, { top, horizon, strength: Math.max(0, number(strength, SKY.strength)) });
    },
    fog(text, color, density) {
      return line(lines.fog, { color, density: Math.max(0, number(density, 0)) });
    },
  };
  apply();

  // The shadow map covers what the camera sees: without perspective,
  // the view between the lowest and highest thing that casts or takes a
  // shadow; with it, a ball round the first REACH tiles of the view. Its
  // size changes only by whole tiles and its middle by whole texels, so
  // a shadow's edge stays where it is while pieces or the camera move.
  const toward = new THREE.Vector3(), right = new THREE.Vector3(), over = new THREE.Vector3(), up = new THREE.Vector3();
  const corner = new THREE.Vector3(), ahead = new THREE.Vector3(), spot = new THREE.Vector3();
  const heights = () => {
    let lo = Infinity, hi = -Infinity;
    scene.traverseVisible((o) => {
      if (!o.isMesh || !(o.castShadow || o.receiveShadow)) return;
      let b = o.boundingBox;
      if (b === undefined || o.isSkinnedMesh) {
        if (!o.geometry.boundingBox) o.geometry.computeBoundingBox();
        b = o.geometry.boundingBox;
      } else if (b === null) {
        o.computeBoundingBox();
        b = o.boundingBox;
      }
      if (!b || b.isEmpty()) return;
      const e = o.matrixWorld.elements;
      let a = e[13], z = e[13];
      for (let i = 0; i < 3; i++) {
        const m = e[1 + 4 * i], p = m * b.min.getComponent(i), q = m * b.max.getComponent(i);
        a += Math.min(p, q);
        z += Math.max(p, q);
      }
      if (Number.isFinite(a) && Number.isFinite(z)) { lo = Math.min(lo, a); hi = Math.max(hi, z); }
    });
    return lo <= hi ? [Math.floor(lo), Math.ceil(hi)] : [0, 1];
  };
  const fit = (camera) => {
    const d = sun.userData.toward;
    toward.set(d[0], d[1], d[2]).normalize();
    up.set(0, 1, 0);
    if (Math.abs(toward.y) > 0.995) up.set(0, 0, -1);
    right.crossVectors(up, toward).normalize();
    over.crossVectors(toward, right);
    let x0 = Infinity, x1 = -Infinity, y0 = Infinity, y1 = -Infinity, z0 = Infinity, z1 = -Infinity;
    const take = (p, r = 0) => {
      const x = p.dot(right), y = p.dot(over), z = p.dot(toward);
      x0 = Math.min(x0, x - r); x1 = Math.max(x1, x + r);
      y0 = Math.min(y0, y - r); y1 = Math.max(y1, y + r);
      z0 = Math.min(z0, z - r); z1 = Math.max(z1, z + r);
    };
    camera.updateMatrixWorld();
    if (camera.isOrthographicCamera) {
      // the view's four edges, cut at the lowest and highest thing
      let [lo, hi] = heights();
      lo = Math.max(lo, look.target[1] - 50);
      hi = Math.max(lo + 1, Math.min(hi, look.target[1] + 50));
      camera.getWorldDirection(ahead);
      const steep = Math.abs(ahead.y) > 0.05;
      for (const sx of [-1, 1]) for (const sy of [-1, 1]) {
        corner.set(sx, sy, -1).unproject(camera);
        for (const y of [lo, hi]) {
          const t = steep ? (y - corner.y) / ahead.y : (y === lo ? 0 : 2 * camera.far);
          take(spot.copy(corner).addScaledVector(ahead, Math.max(-4000, Math.min(4000, t))));
        }
      }
      z1 += (hi - lo) / Math.max(0.15, toward.y);
    } else {
      // a ball round the view as far as REACH: the same size whichever way the camera turns
      const far = Math.min(REACH, camera.far), t = Math.tan(THREE.MathUtils.degToRad((camera.fov || 50) / 2));
      const edge = far * far * t * t * (1 + (camera.aspect || 1) ** 2);
      const along = Math.min(far, (far * far + edge) / (2 * far));
      const r = Math.sqrt(Math.max(edge + (far - along) ** 2, along * along));
      camera.getWorldDirection(ahead);
      take(corner.setFromMatrixPosition(camera.matrixWorld).addScaledVector(ahead, along), r);
      z1 += 2 * REACH;
    }
    const whole = (v) => Math.max(4, Math.ceil(Math.round(v * 1000) / 1000 + 1));
    const w = whole(x1 - x0), h = whole(y1 - y0);
    const sx = w / MAP, sy = h / MAP;
    const cx = Math.round((x0 + x1) / 2 / sx) * sx, cy = Math.round((y0 + y1) / 2 / sy) * sy;
    const near = Math.floor(z0) - 2, far = Math.ceil(z1) + 2;
    const c = sun.shadow.camera;
    // a face the sun grazes is kept out of its own shadow: more, the larger a texel
    sun.shadow.normalBias = 2.5 * Math.max(sx, sy);
    c.up.copy(up);
    if (c.right !== w / 2 || c.top !== h / 2 || c.far !== far - near) {
      c.left = -w / 2; c.right = w / 2; c.bottom = -h / 2; c.top = h / 2;
      c.near = 0; c.far = far - near;
      c.updateProjectionMatrix();
    }
    sun.position.set(0, 0, 0).addScaledVector(right, cx).addScaledVector(over, cy).addScaledVector(toward, far);
    sun.target.position.copy(sun.position).sub(toward);
    sun.updateMatrixWorld();
    sun.target.updateMatrixWorld();
  };

  // With occlusion: the scene into a target with its depths, the
  // corners worked out from the depths (a pixel of the page each), and
  // a last pass to the screen.
  let drawn = null, corners = null, occlusion = null, final = null;
  const behind = new THREE.Color(), was = new THREE.Color();
  if (post) {
    // four samples a pixel, two where a pixel of the page is already four of the screen's
    drawn = new THREE.WebGLRenderTarget(1, 1, { samples: renderer.getPixelRatio() > 1.5 ? 2 : 4, type: THREE.HalfFloatType, depthTexture: new THREE.DepthTexture(1, 1) });
    drawn.depthTexture.minFilter = drawn.depthTexture.magFilter = THREE.NearestFilter;
    corners = new THREE.WebGLRenderTarget(1, 1, { depthBuffer: false });
    occlusion = pass(OCCLUSION, { depths: { value: drawn.depthTexture }, lens: { value: new THREE.Matrix4() }, back: { value: new THREE.Matrix4() }, radius: { value: RADIUS } }, { toneMapped: false });
    final = pass(LAST, { drawn: { value: drawn.texture }, corners: { value: corners.texture }, texel: { value: new THREE.Vector2() }, behind: { value: new THREE.Color() }, dark: { value: DARK } });
  }
  const size = (w, h) => {
    if (!post) return;
    const r = renderer.getPixelRatio();
    drawn.setSize(Math.max(1, Math.floor(w * r)), Math.max(1, Math.floor(h * r)));
    corners.setSize(Math.max(1, w), Math.max(1, h));
    final.material.uniforms.texel.value.set(1 / Math.max(1, w), 1 / Math.max(1, h));
  };

  ctx.render = () => {
    const camera = ctx.camera;
    dome.visible = dome.userData.given && camera.isPerspectiveCamera === true;
    fit(camera);
    if (!post) {
      renderer.render(scene, camera);
      return;
    }
    // the background is laid under the scene by the last pass, as it is
    const back = scene.background;
    if (back && back.isColor) behind.copy(back); else behind.set(look.background);
    renderer.getClearColor(was);
    const alpha = renderer.getClearAlpha();
    scene.background = null;
    renderer.setClearColor(0x000000, 0);
    renderer.setRenderTarget(drawn);
    renderer.render(scene, camera);
    scene.background = back;
    renderer.setClearColor(was, alpha);
    occlusion.material.uniforms.lens.value.copy(camera.projectionMatrix);
    occlusion.material.uniforms.back.value.copy(camera.projectionMatrixInverse);
    renderer.setRenderTarget(corners);
    renderer.render(occlusion, still);
    final.material.uniforms.behind.value.copy(behind).convertLinearToSRGB();
    renderer.setRenderTarget(null);
    renderer.render(final, still);
  };

  return {
    kept,
    size,
    count: () => ({ lights: lines.sun.length + lines.sky.length + lines.fog.length }),
    stop() {
      scene.remove(dome, sun.target);
      dome.material.dispose();
      whole.dispose();
      sun.shadow.dispose();
      scene.fog = null;
      if (!post) return;
      drawn.depthTexture.dispose();
      drawn.dispose();
      corners.dispose();
      occlusion.material.dispose();
      final.material.dispose();
    },
  };
}
