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
// x is east, z south, y up, in tiles. It reads the numbers and decides
// nothing about the game. A scene shown again (a game's tick) changes
// the board in place: a tiles line that is the same text is kept, and
// pieces are matched by model and order.
import * as THREE from 'three';
import { GLTFLoader } from './vendor/three/loaders/GLTFLoader.js';

// how bright a tile's faces are against its top (+x -x +y -y +z -z):
// the south face a little darker, the east face darker still
const SHADES = [0.34, 0.6, 1.0, 0.2, 0.6, 0.34];

// Is this scene a board's (its first call is view)?
export function isBoard(calls) {
  return /^\s*view\(/.test(calls);
}

// Draws the board the calls describe into host (an element); options:
// time (seconds: hold every piece that long after its tick), pixels (a
// cap on the device pixel ratio), models (the folder of the pieces'
// .glb files, './models/' by default; null for plain shapes only),
// keep (the drawing can be read back), onPoint(x, y, event) (a click,
// in the view's units), onFrame. Returns its controls.
export function board(host, calls, options = {}) {
  const scene = new THREE.Scene();
  const camera = new THREE.OrthographicCamera(-1, 1, 1, -1, 0.1, 2000);
  scene.add(new THREE.HemisphereLight(0xffffff, 0x555566, 1.6));
  const sun = new THREE.DirectionalLight(0xffffff, 1.8);
  sun.position.set(0.3, 1.0, 0.8);
  scene.add(sun);
  const look = { w: 0, h: 0, background: '#000000', target: [0, 0, 0], direction: [0.6124, 0.5, 0.6124], scale: 28.284 };
  const ground = new Map(), pieces = [], sources = new Map(), loader = new GLTFLoader();
  const folder = options.models === undefined ? './models/' : options.models;
  const held = options.time !== undefined;
  const box = new THREE.BoxGeometry(1, 1, 1);
  box.translate(0, -0.5, 0);
  const shade = [];
  for (const s of SHADES) for (let i = 0; i < 4; i++) shade.push(s, s, s);
  box.setAttribute('color', new THREE.Float32BufferAttribute(shade, 3));
  const skin = new THREE.MeshLambertMaterial({ color: '#f1c9a5' });
  const torso = new THREE.CapsuleGeometry(0.15, 0.26, 4, 12), head = new THREE.SphereGeometry(0.13, 16, 12), nose = new THREE.BoxGeometry(0.08, 0.08, 0.12);
  let running = true, now = performance.now();

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
    const material = new THREE.MeshBasicMaterial({ color, vertexColors: true });
    const mesh = new THREE.InstancedMesh(box, material, rows.length);
    columns(mesh, rows, (r) => r[2]);
    mesh.name = kind;
    mesh.frustumCulled = false;
    scene.add(mesh);
    const g = { mesh, kind, count: rows.length, shapes: null, gone: false };
    ground.set(line, g);
    if (!model || !rows.length) return;
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
        scene.add(g.shapes);
        columns(mesh, rows, base);
      }, () => {});
    });
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
    return g;
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
        p.body = gltf.scene;
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
  const show = (text) => {
    const wanted = [], kept = new Set(), rest = [];
    for (const line of text.split('\n')) {
      if (!line.trim()) continue;
      if (!line.startsWith('tiles(')) { rest.push(line); continue; }
      kept.add(line);
      if (!ground.has(line)) new Function('nums', 'tiles', line)(nums, (kind, color, rows, model) => tiles(line, kind, color, rows, model));
    }
    for (const [line, g] of ground) {
      if (kept.has(line)) continue;
      clear(g);
      ground.delete(line);
    }
    const api = {
      nums,
      view(w, h, background) { look.w = w; look.h = h; look.background = background; },
      camera(target, direction, scale) { look.target = target; look.direction = direction; look.scale = scale; },
      piece(model, from, clip, t, to, dt) { wanted.push({ model, from, clip, t, to, dt }); },
    };
    new Function(...Object.keys(api), rest.join('\n'))(...Object.values(api));
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
    size();
  };

  const renderer = new THREE.WebGLRenderer({ antialias: true, preserveDrawingBuffer: !!options.keep });
  renderer.setPixelRatio(Math.min(window.devicePixelRatio || 1, options.pixels || 2));
  host.appendChild(renderer.domElement);
  const el = renderer.domElement;
  el.style.display = 'block';
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
    camera.updateProjectionMatrix();
  }
  const gone = new AbortController(), until = { signal: gone.signal };
  window.addEventListener('resize', size, until);
  // a click, in the view's units: what the program's own map turns back into a tile
  el.addEventListener('click', (e) => {
    if (!options.onPoint) return;
    const r = el.getBoundingClientRect();
    options.onPoint((look.w || wide) / 2 + (e.clientX - r.left - wide / 2) / fit, (look.h || high) / 2 + (e.clientY - r.top - high / 2) / fit, e);
  }, until);

  show(calls);

  let shown = 0, rate = 0, since = now;
  const frame = (at) => {
    if (!running) return;
    now = at;
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
    renderer.render(scene, camera);
    shown += 1;
    if (at - since > 500) { rate = shown * 1000 / (at - since); shown = 0; since = at; }
    if (options.onFrame) options.onFrame({ rate, ...count() });
    requestAnimationFrame(frame);
  };
  const count = () => {
    let n = 0;
    for (const g of ground.values()) n += g.count;
    return { tiles: n, kinds: ground.size, modeled: [...ground.values()].filter((g) => g.shapes).length, pieces: pieces.length, models: pieces.filter((p) => p.mixer).length };
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
      for (const g of ground.values()) clear(g);
      for (const g of [box, torso, head, nose]) g.dispose();
      skin.dispose();
      renderer.dispose();
      renderer.forceContextLoss();
      el.remove();
    },
  };
}
