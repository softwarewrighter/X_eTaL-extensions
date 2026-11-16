// The board player: a game's board drawn in real time from the calls a
// program writes, as player.js draws the city's. Four calls, a line
// each, text that names no renderer:
//   view(w, h, background)           the picture's size and what is behind it
//   camera(target, direction, scale) no perspective: looking at target from
//                                    along direction, scale view units a tile
//   tiles(kind, color, nums('x z y ...', 3))
//                                    every tile of a kind: its middle and top
//   tiles(kind, color, nums('x z y base ...', 4), model)
//                                    the same drawn with a model at each tile
//   piece(model, nums('x z y hx hz', 5)[0], clip, t, nums('x z y', 3)[0], dt)
//                                    a piece: where it is and the way it
//                                    faces, its clip and the time into it,
//                                    and where it will be in dt seconds
// and the calls of its modules (board/*.js): light's are sun, sky and
// fog. x is east, z south, y up, in tiles. It reads the numbers and decides
// nothing about the game. A scene shown again (a game's tick) changes
// the board in place: a tiles line that is the same text is kept, and
// pieces are matched by model and order.
import * as THREE from 'three';
import { GLTFLoader } from './vendor/three/loaders/GLTFLoader.js';
import light, { FRONT } from './board/light.js';
import terrain from './board/terrain.js';
import water from './board/water.js';
import particles from './board/particles.js';
import eye from './board/eye.js';

// What draws the calls this file does not: a module a group of calls.
// A module is a function of the board's context giving any of
//   kept:  { name(line, ...args) }  calls kept by their line: made once
//          when a scene first holds that line of text, left alone while
//          every later scene holds the same line, and disposed (the
//          object returned, if it has dispose()) when a scene no longer
//          does. Lines gone are disposed before new ones are made.
//   each:  { name(...args) }        calls run every time a scene is shown
//   begin(), end()                  round each scene shown
//   frame(time, dt)                 before each picture: the board's
//                                   clock in seconds (options.time when
//                                   held) and the seconds since the last
//   size(wide, high)                when the picture's size changes
//   count()                         numbers to add to the board's count
//   stop()                          the board is closed
// The context: THREE, scene, renderer, host, options, look (the view's
// and camera's numbers), nums, ints, folder, loader, source(model) (a
// .glb's bytes, fetched once), flat (the board's own camera, without
// perspective), view() (wide, high, fit: the element's size and the
// pixels a view unit), until (an options object for addEventListener,
// ended when the board closes), and three things a module may set:
// camera (what the picture is taken with), render() (takes the
// picture; by default renderer.render(scene, camera)), lights (sky and
// sun, the board's two lights, there until a module changes them).
// The light module (board/light.js) owns the lights, the shadows, fog
// (scene.fog) and tone mapping, and takes the picture (it sets
// render()): a module's meshes use Three's own lit materials and say
// castShadow and receiveShadow, and are lit, shadowed and fogged with
// the tiles and pieces. The board's own camera stands 500 from its
// target; its near plane is kept FRONT behind the front of the picture
// (the ground at the bottom middle of the view), where fog starts.
const MODULES = [light, terrain, water, particles, eye];

// Is this scene a board's (its first call is view)?
export function isBoard(calls) {
  return /^\s*view\(/.test(calls);
}

// Draws the board the calls describe into host (an element); options:
// time (seconds: hold every piece that long after its tick), pixels (a
// cap on the device pixel ratio), models (the folder of the pieces'
// .glb files, './models/' by default; null for plain shapes only),
// keep (the drawing can be read back), onPoint(x, y, event) (a click,
// in the view's units), onFrame, occlusion (false: no ambient
// occlusion, the picture drawn straight to the screen). Returns its
// controls.
export function board(host, calls, options = {}) {
  const scene = new THREE.Scene();
  const camera = new THREE.OrthographicCamera(-1, 1, 1, -1, 0.1, 2000);
  // the board's two lights: what they are is the light module's to say
  const sky = new THREE.HemisphereLight(), sun = new THREE.DirectionalLight();
  scene.add(sky, sun);
  const look = { w: 0, h: 0, background: '#000000', target: [0, 0, 0], direction: [0.6124, 0.5, 0.6124], scale: 28.284 };
  const ground = new Map(), pieces = [], sources = new Map(), loader = new GLTFLoader(), modules = [];
  const folder = options.models === undefined ? './models/' : options.models;
  const held = options.time !== undefined;
  const box = new THREE.BoxGeometry(1, 1, 1);
  box.translate(0, -0.5, 0);
  const skin = new THREE.MeshLambertMaterial({ color: '#f1c9a5' });
  const torso = new THREE.CapsuleGeometry(0.15, 0.26, 4, 12), head = new THREE.SphereGeometry(0.13, 16, 12), nose = new THREE.BoxGeometry(0.08, 0.08, 0.12);
  let running = true, now = performance.now();

  // everything drawn is lit by the board's lights, and casts and takes shadows
  const lit = (root) => { root.traverse((o) => { if (o.isMesh) { o.castShadow = true; o.receiveShadow = true; } }); return root; };

  const nums = (text, k) => {
    const v = text.trim() ? text.trim().split(/\s+/).map(Number) : [], rows = [];
    for (let i = 0; i < v.length; i += k) rows.push(v.slice(i, i + k));
    return rows;
  };

  // one kind's tiles: a column each, from the ground to its top (y); with
  // a model, the model at each tile once it has come, on a column to its
  // base (a row's fourth number; its top when there is none), turned to
  // face a row's fifth and sixth
  const columns = (mesh, rows, to) => {
    const m = new THREE.Matrix4(), at = new THREE.Vector3(), turn = new THREE.Quaternion(), tall = new THREE.Vector3();
    rows.forEach((r, i) => {
      const y = to(r);
      at.set(r[0], Math.max(y, 0), r[1]);
      tall.set(1, Math.max(y, 0.001), 1);
      mesh.setMatrixAt(i, m.compose(at, turn, tall));
    });
    mesh.instanceMatrix.needsUpdate = true;
  };
  const tiles = (line, kind, color, rows, model) => {
    const material = new THREE.MeshLambertMaterial({ color });
    const mesh = new THREE.InstancedMesh(box, material, rows.length);
    columns(mesh, rows, (r) => r[2]);
    mesh.name = kind;
    mesh.frustumCulled = false;
    scene.add(lit(mesh));
    const g = { mesh, kind, count: rows.length, shapes: null, gone: false, dispose() { clear(g); } };
    if (!model || !rows.length) return g;
    source(model).then((bytes) => {
      if (!bytes || g.gone || !running) return;
      loader.parse(bytes.slice(0), folder, (gltf) => {
        if (g.gone || !running) return;
        const base = (r) => (r.length > 3 ? r[3] : r[2]);
        const place = new THREE.Matrix4(), at = new THREE.Vector3(), turn = new THREE.Quaternion(), one = new THREE.Vector3(1, 1, 1), up = new THREE.Vector3(0, 1, 0);
        g.shapes = new THREE.Group();
        gltf.scene.updateMatrixWorld(true);
        gltf.scene.traverse((o) => {
          if (!o.isMesh) return;
          const part = new THREE.InstancedMesh(o.geometry, o.material, rows.length);
          rows.forEach((r, i) => {
            at.set(r[0], base(r), r[1]);
            turn.setFromAxisAngle(up, r.length > 5 ? Math.atan2(r[4], r[5]) : 0);
            part.setMatrixAt(i, place.compose(at, turn, one).multiply(o.matrixWorld));
          });
          part.frustumCulled = false;
          g.shapes.add(part);
        });
        scene.add(lit(g.shapes));
        columns(mesh, rows, base);
      }, () => {});
    });
    return g;
  };
  const clear = (g) => {
    g.gone = true;
    scene.remove(g.mesh);
    g.mesh.material.dispose();
    g.mesh.dispose();
    if (!g.shapes) return;
    scene.remove(g.shapes);
    g.shapes.traverse((o) => { if (o.isMesh) { o.geometry.dispose(); [].concat(o.material).forEach((x) => x.dispose()); o.dispose(); } });
  };

  // a piece with no model (yet): a body, a head, and a nose the way it faces
  const plain = (model) => {
    let hash = 0;
    for (const c of model) hash = (hash * 31 + c.charCodeAt(0)) % 360;
    const g = new THREE.Group();
    const coat = new THREE.MeshLambertMaterial({ color: new THREE.Color().setHSL(hash / 360, 0.6, 0.45) });
    const body = new THREE.Mesh(torso, coat), top = new THREE.Mesh(head, skin), front = new THREE.Mesh(nose, skin);
    body.position.y = 0.28;
    top.position.y = 0.68;
    front.position.set(0, 0.66, 0.14);
    g.add(body, top, front);
    g.userData.coat = coat;
    return lit(g);
  };

  // a model's file, fetched once however many pieces use it; null when there is none
  const source = (model) => {
    if (!sources.has(model)) {
      sources.set(model, folder === null ? Promise.resolve(null)
        : fetch(`${folder}${model}.glb`).then((r) => (r.ok ? r.arrayBuffer() : null)).catch(() => null));
    }
    return sources.get(model);
  };

  const pose = (p) => {
    if (!p.mixer) return;
    if (p.playing !== p.clip) {
      if (p.action) p.action.stop();
      const clip = THREE.AnimationClip.findByName(p.clips, p.clip);
      p.action = clip ? p.mixer.clipAction(clip) : null;
      if (p.action) p.action.play();
      p.playing = p.clip;
    }
  };

  const add = (model) => {
    const p = { model, root: new THREE.Group(), body: plain(model), mixer: null, clips: [], action: null, playing: null, gone: false };
    p.root.add(p.body);
    scene.add(p.root);
    source(model).then((bytes) => {
      if (!bytes || p.gone || !running) return;
      loader.parse(bytes.slice(0), folder, (gltf) => {
        if (p.gone || !running) return;
        p.root.remove(p.body);
        p.body.userData.coat.dispose();
        p.body = lit(gltf.scene);
        p.root.add(p.body);
        p.mixer = new THREE.AnimationMixer(p.body);
        p.clips = gltf.animations;
        pose(p);
      }, () => {});
    });
    return p;
  };

  const drop = (p) => {
    p.gone = true;
    scene.remove(p.root);
    p.root.traverse((o) => { if (o.geometry && ![torso, head, nose].includes(o.geometry)) o.geometry.dispose(); if (o.material && o.material !== skin) o.material.dispose(); });
  };

  // the calls of one scene, in place of the one before
  const wanted = [];
  const each = {
    view(w, h, background) { look.w = w; look.h = h; look.background = background; },
    camera(target, direction, scale) { look.target = target; look.direction = direction; look.scale = scale; ctx.camera = camera; },
    piece(model, from, clip, t, to, dt) { wanted.push({ model, from, clip, t, to, dt }); },
  };
  const kept = { tiles };
  const show = (text) => {
    const lines = text.split('\n').filter((l) => l.trim()), seen = new Set();
    const name = (line) => line.slice(0, line.indexOf('(')).trim();
    for (const line of lines) if (kept[name(line)]) seen.add(line);
    for (const [line, made] of ground) {
      if (seen.has(line)) continue;
      if (made && made.dispose) made.dispose();
      ground.delete(line);
    }
    wanted.length = 0;
    for (const m of modules) if (m.begin) m.begin();
    // a line is one call: a kept one is made only when its text is new
    let current = '';
    const api = { nums, ints: nums, ...each };
    for (const k of Object.keys(kept)) api[k] = (...args) => { ground.set(current, kept[k](current, ...args) || {}); };
    const names = Object.keys(api), values = Object.values(api);
    for (const line of lines) {
      if (ground.has(line)) continue;
      current = line;
      new Function(...names, line)(...values);
    }
    // the i-th piece of a model is the piece it was in the scene before
    const had = new Map(), next = [];
    for (const p of pieces) { if (!had.has(p.model)) had.set(p.model, []); had.get(p.model).push(p); }
    for (const w of wanted) {
      const old = had.get(w.model);
      const p = old && old.length ? old.shift() : add(w.model);
      Object.assign(p, { from: w.from, clip: w.clip, t: w.t, to: w.to, dt: w.dt, since: now });
      pose(p);
      next.push(p);
    }
    for (const old of had.values()) old.forEach(drop);
    pieces.length = 0;
    pieces.push(...next);
    scene.background = new THREE.Color(look.background);
    for (const m of modules) if (m.end) m.end();
    size();
  };

  const renderer = new THREE.WebGLRenderer({ antialias: true, preserveDrawingBuffer: !!options.keep });
  renderer.setPixelRatio(Math.min(window.devicePixelRatio || 1, options.pixels || 2));
  host.appendChild(renderer.domElement);
  const el = renderer.domElement;
  el.style.display = 'block';
  const gone = new AbortController(), until = { signal: gone.signal };
  const ctx = {
    THREE, scene, renderer, host, options, look, nums, ints: nums, folder, loader, source, flat: camera, until,
    view: () => ({ wide, high, fit }),
    camera,
    lights: { sky, sun },
    render() { renderer.render(scene, ctx.camera); },
  };
  for (const m of MODULES) {
    const made = m(ctx) || {};
    for (const [k, f] of Object.entries(made.kept || {})) kept[k] = f;
    for (const [k, f] of Object.entries(made.each || {})) each[k] = f;
    modules.push(made);
  }
  // the whole view, as large as the element lets it be; a view unit is fit pixels
  let fit = 1, wide = 1, high = 1;
  function size() {
    wide = host.clientWidth || look.w || 1280;
    high = host.clientHeight || look.h || 720;
    renderer.setSize(wide, high);
    fit = look.w && look.h ? Math.min(wide / look.w, high / look.h) : 1;
    const half = 0.5 / (fit * look.scale);
    camera.left = -wide * half; camera.right = wide * half; camera.top = high * half; camera.bottom = -high * half;
    const [tx, ty, tz] = look.target, [dx, dy, dz] = look.direction;
    camera.position.set(tx + dx * 500, ty + dy * 500, tz + dz * 500);
    camera.lookAt(tx, ty, tz);
    // the front of the picture (the ground at the bottom middle of the
    // view) is FRONT past the near plane: where the light module's fog starts
    const level = Math.hypot(dx, dz), long = Math.hypot(dx, dy, dz) || 1;
    const front = dy > 0 ? Math.min(250, camera.top * level / dy) : 0;
    camera.near = Math.max(0.1, 500 * long - front - FRONT);
    camera.updateProjectionMatrix();
    for (const m of modules) if (m.size) m.size(wide, high);
  }
  window.addEventListener('resize', size, until);
  // a click, in the view's units: what the program's own map turns back into a tile
  el.addEventListener('click', (e) => {
    // only the board's own camera: another's picture is not in the view's units
    if (!options.onPoint || ctx.camera !== camera) return;
    const r = el.getBoundingClientRect();
    options.onPoint((look.w || wide) / 2 + (e.clientX - r.left - wide / 2) / fit, (look.h || high) / 2 + (e.clientY - r.top - high / 2) / fit, e);
  }, until);

  show(calls);

  let shown = 0, rate = 0, since = now, clock = 0, before = now;
  const frame = (at) => {
    if (!running) return;
    const dt = Math.min(0.1, Math.max(0, (at - before) / 1000));
    before = at;
    now = at;
    clock = held ? options.time : clock + dt;
    for (const p of pieces) {
      // from where it is to where it will be, over dt seconds from its tick
      const passed = held ? options.time : (at - p.since) / 1000;
      const u = p.dt > 0 ? Math.min(1, Math.max(0, passed / p.dt)) : 1;
      p.root.position.set(p.from[0] + (p.to[0] - p.from[0]) * u, p.from[2] + (p.to[2] - p.from[2]) * u, p.from[1] + (p.to[1] - p.from[1]) * u);
      p.root.rotation.y = Math.atan2(p.from[3], p.from[4]);
      if (p.action) {
        const long = p.action.getClip().duration;
        p.action.time = long > 0 ? (p.t + passed) % long : 0;
        p.mixer.update(0);
      }
    }
    for (const m of modules) if (m.frame) m.frame(clock, held ? 0 : dt);
    ctx.render();
    shown += 1;
    if (at - since > 500) { rate = shown * 1000 / (at - since); shown = 0; since = at; }
    if (options.onFrame) options.onFrame({ rate, ...count() });
    requestAnimationFrame(frame);
  };
  const count = () => {
    const kinds = [...ground.values()].filter((g) => g.mesh);
    let n = 0;
    for (const g of kinds) n += g.count;
    const all = { tiles: n, kinds: kinds.length, modeled: kinds.filter((g) => g.shapes).length, pieces: pieces.length, models: pieces.filter((p) => p.mixer).length };
    for (const m of modules) if (m.count) Object.assign(all, m.count());
    return all;
  };
  requestAnimationFrame(frame);

  return {
    board: true,
    show,
    count,
    stop() {
      running = false;
      gone.abort();
      pieces.forEach(drop);
      for (const m of modules) if (m.stop) m.stop();
      for (const made of ground.values()) if (made && made.dispose) made.dispose();
      for (const g of [box, torso, head, nose]) g.dispose();
      skin.dispose();
      renderer.dispose();
      renderer.forceContextLoss();
      el.remove();
    },
  };
}
