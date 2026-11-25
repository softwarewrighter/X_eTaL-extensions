// The cell player: an assembly cell's timeline drawn and played with
// Three.js, from the calls the cell extension writes (ce:s_cene), one a
// line, as the city's and the board's players read theirs:
//   cell(name, w, h)                    the scene's first call: this player
//   fixture(id, model, pose)            a fixture's model at a pose
//   rig(id, model, base)                an arm's model at its base
//   track(id, rows)                     an arm's joints over time: rows of
//                                       t j1 .. j6; id 'stage': t angle
//   part(id, kind, pose)                a part where it starts
//   attach(id, arm, t0, t1)             the part (or tool) moves with the
//                                       arm's flange from t0 to t1, then
//                                       stays; arm 'stage': the vise holds
//                                       it, turning with the stage
//   place(id, t, pose)                  from t the part is there (where a
//                                       fall ended)
//   ghost(id, kind, pose)               where a part ends: drawn faint,
//                                       before it is there
//   step(n, label, t0, t1)              a command, for the list of steps
//   expect(id, t, xyz)                  where an arm's flange (or a part's
//                                       origin) is at t, for the check
//                                       (?check)
// A pose is x y z (mm) and roll, pitch, yaw (degrees, Rz Ry Rx), Z up:
// the cell's frame, as the simulator's. Everything hangs from one
// group turned -90 degrees about x, so the scene is Three.js's Y up.
// The models are the cell extension's .glb files (CellModels.xtl), in
// meters and Y up as glTF has them, hung under a group that makes them
// millimeters and Z up again; an arm's joint is its node arm_jN, whose
// Blender z (the joint's axis) glTF made its local y.
//
// window.cell, as console-3D's window.selectric: await cell.play(from,
// to) plays (resolving at to), cell.seek(t), cell.state (the time,
// whether playing, the speed, each arm's joints and flange, the stage,
// the step), cell.stop(). ?t=SECONDS holds that moment; ?check seeks
// every expect and writes the result into #check (for headless Chrome:
// --dump-dom); ?hud=0 hides the controls.
import * as THREE from 'three';
import { GLTFLoader } from './vendor/three/loaders/GLTFLoader.js';

// Is this scene a cell's (its first call is cell)?
export function isCell(calls) {
  return /^\s*cell\(/.test(calls);
}

const D = Math.PI / 180;

// the models, each .glb fetched and parsed once for every player on
// the page (a live page makes a new player for each command)
const loader = new GLTFLoader();
const sources = {};

function nums(text, k) {
  const v = text.trim() ? text.trim().split(/\s+/).map(Number) : [];
  const rows = [];
  for (let i = 0; i + k <= v.length; i += k) rows.push(v.slice(i, i + k));
  return rows;
}

// a pose (x y z r p y) as a matrix in the cell's frame
function poseMatrix(p) {
  const m = new THREE.Matrix4().makeRotationFromEuler(new THREE.Euler(p[3] * D, p[4] * D, p[5] * D, 'ZYX'));
  m.setPosition(p[0], p[1], p[2]);
  return m;
}

// a track's values at time t: rows interpolated linearly, held at the ends
function at(rows, t) {
  if (!rows || !rows.length) return null;
  if (t <= rows[0][0]) return rows[0].slice(1);
  const last = rows[rows.length - 1];
  if (t >= last[0]) return last.slice(1);
  let lo = 0, hi = rows.length - 1;
  while (hi - lo > 1) { const mid = (lo + hi) >> 1; if (rows[mid][0] <= t) lo = mid; else hi = mid; }
  const a = rows[lo], b = rows[hi], k = (t - a[0]) / (b[0] - a[0]);
  return a.slice(1).map((x, i) => x + (b[i + 1] - x) * k);
}

// Plays the cell the calls describe in host (an element); options: time
// (hold that moment), models (the folder of the .glb files, './models/'),
// hud (false: no controls), check (seek every expect, report into #check),
// onFrame. Returns its controls (also window.cell).
export function cellPlayer(host, calls, options = {}) {
  const scene = new THREE.Scene();
  scene.background = new THREE.Color('#202329');
  // the window's listeners end with the player (a live page makes a new
  // player for each command)
  const stopper = new AbortController();
  const ends = { signal: stopper.signal };
  const root = new THREE.Group();
  root.rotation.x = -Math.PI / 2;
  scene.add(root);
  const folder = options.models || './models/';
  const s = {
    name: 'cell', w: 1280, h: 720, fixtures: [], rigs: [], tracks: {}, parts: [], attaches: [], places: [], ghosts: [], steps: [], expects: [],
  };
  const api = {
    nums,
    cell(name, w, h) { s.name = name; s.w = w; s.h = h; },
    fixture(id, model, pose) { s.fixtures.push({ id, model, pose }); },
    rig(id, model, base) { s.rigs.push({ id, model, base }); },
    track(id, rows) { s.tracks[id] = rows; },
    part(id, kind, pose) { s.parts.push({ id, kind, pose }); },
    attach(id, arm, t0, t1) { s.attaches.push({ id, arm, t0, t1 }); },
    place(id, t, pose) { s.places.push({ id, t, pose }); },
    ghost(id, kind, pose) { s.ghosts.push({ id, kind, pose }); },
    step(n, label, t0, t1) { s.steps.push({ n, label, t0, t1 }); },
    expect(arm, t, xyz) { s.expects.push({ arm, t, xyz }); },
  };
  new Function(...Object.keys(api), calls)(...Object.values(api));
  const duration = Math.max(0, ...s.steps.map((x) => x.t1), ...s.places.map((x) => x.t), ...Object.values(s.tracks).map((r) => (r.length ? r[r.length - 1][0] : 0)));

  // light, floor, camera
  scene.add(new THREE.HemisphereLight('#dfe6f0', '#3a3530', 1.6));
  const sun = new THREE.DirectionalLight('#ffffff', 2.2);
  sun.position.set(600, 1200, 800);
  scene.add(sun);
  const floor = new THREE.Mesh(new THREE.CircleGeometry(900, 64), new THREE.MeshStandardMaterial({ color: '#2c3038', roughness: 0.9 }));
  floor.position.z = -200.5;
  root.add(floor);
  const grid = new THREE.GridHelper(1800, 36, '#4a505c', '#383d47');
  grid.rotation.x = Math.PI / 2;
  grid.position.z = -200;
  root.add(grid);
  const camera = new THREE.PerspectiveCamera(40, 16 / 9, 5, 10000);
  // where the camera looks from: options.view (a player's view before),
  // else a three-quarter view of the whole cell
  const view = options.view || { yaw: 35 * D, pitch: 28 * D, dist: 1700, target: new THREE.Vector3(0, 60, 0) };
  const place = () => {
    const c = Math.cos(view.pitch);
    camera.position.set(view.target.x + view.dist * c * Math.sin(view.yaw), view.target.y + view.dist * Math.sin(view.pitch), view.target.z + view.dist * c * Math.cos(view.yaw));
    camera.lookAt(view.target);
  };
  place();

  const renderer = new THREE.WebGLRenderer({ antialias: true, preserveDrawingBuffer: true });
  renderer.setPixelRatio(Math.min(window.devicePixelRatio || 1, 2));
  host.appendChild(renderer.domElement);
  renderer.domElement.style.display = 'block';
  const size = () => {
    const w = host.clientWidth || window.innerWidth, h = host.clientHeight || window.innerHeight;
    renderer.setSize(w, h);
    camera.aspect = w / h;
    camera.updateProjectionMatrix();
  };
  size();
  window.addEventListener('resize', size, ends);

  // dragging turns the view, the wheel comes closer
  let drag = null;
  renderer.domElement.addEventListener('pointerdown', (e) => { drag = [e.clientX, e.clientY]; });
  window.addEventListener('pointerup', () => { drag = null; }, ends);
  window.addEventListener('pointermove', (e) => {
    if (!drag) return;
    view.yaw -= (e.clientX - drag[0]) * 0.005;
    view.pitch = Math.max(-0.2, Math.min(1.5, view.pitch + (e.clientY - drag[1]) * 0.005));
    drag = [e.clientX, e.clientY];
    place();
    draw();
  }, ends);
  renderer.domElement.addEventListener('wheel', (e) => { e.preventDefault(); view.dist = Math.max(200, Math.min(5000, view.dist * Math.exp(e.deltaY * 0.001))); place(); draw(); }, { passive: false });

  // the models: each .glb fetched once, a copy for each use, hung under
  // a group that turns glTF's meters and Y up into the cell's mm and Z up
  const load = (model) => (sources[folder + model] ||= new Promise((ok, no) => loader.load(folder + model + '.glb', (g) => ok(g.scene), undefined, () => no(new Error('no model ' + model + '.glb')))));
  const holder = (obj) => {
    const g = new THREE.Group();
    const inner = new THREE.Group();
    inner.rotation.x = Math.PI / 2;
    inner.scale.setScalar(1000);
    inner.add(obj);
    g.add(inner);
    return g;
  };
  const arms = {}, fixtures = {}, parts = {};
  const Y = new THREE.Vector3(0, 1, 0);
  const ready = Promise.all([
    ...s.fixtures.map(async (f) => {
      const g = holder((await load(f.model)).clone(true));
      g.matrixAutoUpdate = false;
      g.matrix.copy(poseMatrix(f.pose));
      root.add(g);
      fixtures[f.id] = { g, turn: g.getObjectByName('stage_turn') };
      if (fixtures[f.id].turn) fixtures[f.id].rest = fixtures[f.id].turn.quaternion.clone();
    }),
    ...s.rigs.map(async (r) => {
      const g = holder((await load(r.model)).clone(true));
      g.matrixAutoUpdate = false;
      g.matrix.copy(poseMatrix(r.base));
      root.add(g);
      const joints = [1, 2, 3, 4, 5, 6].map((n) => g.getObjectByName('arm_j' + n));
      if (joints.some((j) => !j)) throw new Error(r.model + '.glb has no arm_j1 to arm_j6');
      arms[r.id] = { g, joints, rest: joints.map((j) => j.quaternion.clone()), flange: g.getObjectByName('arm_flange'), q: [0, 0, 0, 0, 0, 0] };
    }),
    ...s.parts.map(async (p) => {
      const g = holder((await load(p.kind)).clone(true));
      g.matrixAutoUpdate = false;
      root.add(g);
      parts[p.id] = { g, start: poseMatrix(p.pose) };
    }),
    ...s.ghosts.map(async (p) => {
      const g = holder((await load(p.kind)).clone(true));
      g.traverse((o) => { if (o.isMesh) o.material = new THREE.MeshBasicMaterial({ color: '#7fd4ff', transparent: true, opacity: 0.22, depthWrite: false }); });
      g.matrixAutoUpdate = false;
      g.matrix.copy(poseMatrix(p.pose));
      g.name = 'ghost';
      root.add(g);
    }),
  ]);

  // the cell at time t
  const pose = (arm, q) => {
    const a = arms[arm];
    a.q = q;
    a.joints.forEach((j, i) => j.quaternion.copy(a.rest[i]).multiply(new THREE.Quaternion().setFromAxisAngle(Y, q[i] * D)));
  };
  const flangeIn = (arm) => {
    root.updateMatrixWorld(true);
    const inv = new THREE.Matrix4().copy(root.matrixWorld).invert();
    return inv.multiply(arms[arm].flange.matrixWorld.clone());
  };
  // a part's matrix at t: where it starts, or carried by an arm
  // what carries a part at time u: an arm's flange, or the stage's top
  const carrier = (by, u) => {
    if (by === 'stage') {
      turnStage(u);
      root.updateMatrixWorld(true);
      return new THREE.Matrix4().copy(root.matrixWorld).invert().multiply(fixtures.stage.turn.matrixWorld.clone());
    }
    pose(by, at(s.tracks[by], u));
    return flangeIn(by);
  };
  // a part's matrix at t: where it starts, carried while held, put
  // where a fall ended -- each event in time order
  const partAt = (id, t) => {
    let m = parts[id].start.clone();
    const events = [
      ...s.attaches.filter((x) => x.id === id && x.t0 <= t).map((a) => ({ t: a.t0, a })),
      ...s.places.filter((x) => x.id === id && x.t <= t).map((p) => ({ t: p.t, p })),
    ].sort((x, y) => x.t - y.t);
    for (const e of events) {
      if (e.p) { m = poseMatrix(e.p.pose); continue; }
      const a = e.a;
      if (a.arm !== 'stage' && !arms[a.arm]) continue;
      if (a.arm === 'stage' && !(fixtures.stage && fixtures.stage.turn)) continue;
      const offset = carrier(a.arm, a.t0).invert().multiply(m);
      m = carrier(a.arm, Math.min(t, a.t1)).multiply(offset);
    }
    return m;
  };
  const turnStage = (u) => {
    const st = fixtures.stage;
    if (st && st.turn) st.turn.quaternion.copy(st.rest).multiply(new THREE.Quaternion().setFromAxisAngle(Y, (at(s.tracks.stage, u) || [0])[0] * D));
  };
  let now = 0;
  const seek = (t) => {
    now = Math.max(0, Math.min(duration, t));
    for (const id of Object.keys(parts)) parts[id].g.matrix.copy(partAt(id, now));
    for (const id of Object.keys(arms)) pose(id, at(s.tracks[id], now) || [0, 0, 0, 0, 90, 0]);
    turnStage(now);
    ui.show();
    draw();
  };
  const draw = () => renderer.render(scene, camera);

  // playing: one loop, every frame -- the time moves on while playing,
  // and the picture is drawn each frame (a picture drawn once is not
  // always shown, as in headless Chrome)
  let playing = false, speed = 1, until = null, last = 0, running = true;
  const tick = (ms) => {
    if (!running) return;
    if (playing) {
      const dt = last ? (ms - last) / 1000 : 0;
      last = ms;
      let t = now + dt * speed;
      if (until && t >= until.to) { t = until.to; playing = false; }
      if (t >= duration) { t = duration; playing = false; }
      seek(t);
      if (!playing) { last = 0; if (until) { const u = until; until = null; u.done(); } ui.show(); }
    }
    draw();
    requestAnimationFrame(tick);
  };
  const controls = {
    play(from, to) {
      return ready.then(() => new Promise((done) => {
        if (from !== undefined) seek(from);
        if (until) until.done();
        until = to !== undefined ? { to, done } : { to: duration, done };
        playing = true; last = 0;
      }));
    },
    seek(t) { seek(t); },
    stop() { playing = false; last = 0; if (until) { const u = until; until = null; u.done(); } ui.show(); },
    get state() {
      const out = { name: s.name, t: now, duration, playing, speed, arms: {}, stage: (at(s.tracks.stage, now) || [0])[0], step: stepAt(now) };
      for (const id of Object.keys(arms)) {
        const f = new THREE.Vector3().setFromMatrixPosition(flangeIn(id));
        out.arms[id] = { joints: arms[id].q.map((x) => +x.toFixed(4)), flange: [f.x, f.y, f.z].map((x) => +x.toFixed(3)) };
      }
      out.parts = {};
      for (const id of Object.keys(parts)) {
        const p = new THREE.Vector3().setFromMatrixPosition(parts[id].g.matrix);
        out.parts[id] = [p.x, p.y, p.z].map((x) => +x.toFixed(3));
      }
      return out;
    },
    set speed(v) { speed = v; ui.show(); },
    get view() { return view; },
    // stepping through the list, every item in turn, as a debugger
    // steps: Next plays the next command from its start to its end;
    // Previous goes back to the start of this command (of the one before
    // when already at a start). Steps that start at the same time (a
    // reset, then a tool taken) are still each their own.
    next() {
      const k = cursor() + 1;
      if (k >= s.steps.length) return Promise.resolve();
      const step = s.steps[k];
      controls.stop();
      seek(step.t0);
      pinned = k;
      ui.show();
      return controls.play(step.t0, step.t1).then(() => { pinned = k; ui.show(); });
    },
    previous() {
      let k = cursor();
      if (k < 0) return;
      if (now <= s.steps[k].t0 + 1e-6) k -= 1;
      controls.stop();
      if (k < 0) { pinned = null; seek(0); return; }
      seek(s.steps[k].t0);
      pinned = k;
      ui.show();
    },
    get duration() { return duration; },
  };
  // which step the list shows: the one stepped to (until the time moves
  // by other means), else the last to have started
  let pinned = null;
  const cursor = () => {
    if (pinned !== null && s.steps[pinned] && now >= s.steps[pinned].t0 - 1e-6 && now <= s.steps[pinned].t1 + 1e-6) return pinned;
    pinned = null;
    // a step under way (strictly inside its times), else the last done;
    // a step that starts now has not begun
    const going = s.steps.findIndex((x) => x.t0 < now - 1e-9 && now < x.t1 - 1e-9);
    if (going >= 0) return going;
    let k = -1;
    s.steps.forEach((x, i) => { if (x.t1 <= now + 1e-9) k = i; });
    return k;
  };
  const stepAt = () => { const k = cursor(); return k >= 0 ? s.steps[k].n : 0; };

  // the controls: play, time, scrubber, speed, the steps
  const ui = (() => {
    const box = document.createElement('div');
    box.id = 'cellbar';
    box.style.cssText = 'position:fixed;left:12px;right:12px;bottom:12px;background:rgba(8,9,14,0.82);color:#e8eaf0;padding:8px 12px;border-radius:8px;font:14px/1.4 system-ui,sans-serif;display:flex;gap:10px;align-items:center';
    const play = document.createElement('button');
    const time = document.createElement('span');
    time.style.cssText = 'font-variant-numeric:tabular-nums;min-width:9em';
    const bar = document.createElement('input');
    Object.assign(bar, { type: 'range', min: 0, max: duration, step: 0.01, value: 0 });
    bar.style.flex = '1';
    const rate = document.createElement('select');
    for (const v of [0.25, 0.5, 1, 2, 4]) rate.add(new Option(v + 'x', v, v === 1, v === 1));
    // step by step through the list: the time to the next step's start,
    // or back to this step's start (the one before, when already there)
    const back = document.createElement('button');
    back.textContent = 'Previous step';
    const ahead = document.createElement('button');
    ahead.textContent = 'Next step';
    const ghosts = document.createElement('label');
    ghosts.innerHTML = '<input type="checkbox" checked> ghosts';
    for (const b of [play, back, ahead]) b.style.cssText = 'font:inherit;padding:3px 12px;border-radius:6px;border:1px solid #56607a;background:#1a1f2e;color:#e8eaf0;cursor:pointer';
    box.append(play, back, ahead, time, bar, rate, ghosts);
    const list = document.createElement('ol');
    list.id = 'cellsteps';
    list.style.cssText = 'position:fixed;right:12px;top:12px;max-height:60vh;overflow:auto;margin:0;background:rgba(8,9,14,0.82);color:#e8eaf0;padding:8px 12px 8px 36px;border-radius:8px;font:13px/1.5 ui-monospace,Menlo,monospace;max-width:40vw';
    s.steps.forEach((k, i) => {
      const li = document.createElement('li');
      li.textContent = k.label;
      li.title = `${k.t0.toFixed(2)} to ${k.t1.toFixed(2)} s: click to go there; Next step plays it`;
      li.style.cursor = 'pointer';
      li.addEventListener('click', () => { controls.stop(); seek(k.t0); pinned = i; ui.show(); });
      list.append(li);
    });
    if (options.hud !== false) document.body.append(box, list);
    play.addEventListener('click', () => (playing ? controls.stop() : controls.play(now >= duration ? 0 : undefined)));
    back.addEventListener('click', () => controls.previous());
    ahead.addEventListener('click', () => controls.next());
    // the arrow keys step too, unless a field has the keys
    window.addEventListener('keydown', (e) => {
      if (/^(INPUT|TEXTAREA|SELECT)$/.test((e.target && e.target.tagName) || '')) return;
      if (e.key === 'ArrowRight') { e.preventDefault(); controls.next(); }
      if (e.key === 'ArrowLeft') { e.preventDefault(); controls.previous(); }
    }, ends);
    bar.addEventListener('input', () => { controls.stop(); seek(Number(bar.value)); });
    rate.addEventListener('change', () => { speed = Number(rate.value); });
    ghosts.firstChild.addEventListener('change', (e) => { root.children.filter((c) => c.name === 'ghost').forEach((g) => { g.visible = e.target.checked; }); draw(); });
    let shown = 0;
    return {
      show() {
        play.textContent = playing ? 'Pause' : 'Play';
        time.textContent = `${now.toFixed(2)} / ${duration.toFixed(2)} s`;
        bar.value = now;
        const n = stepAt(now);
        if (n !== shown) {
          [...list.children].forEach((li, i) => { li.style.color = i + 1 === n ? '#ffd27f' : ''; });
          shown = n;
        }
        if (options.onFrame) options.onFrame(controls.state);
      },
      remove() { box.remove(); list.remove(); },
    };
  })();

  ready.then(() => {
    seek(options.time !== undefined ? options.time : 0);
    requestAnimationFrame(tick);
    if (options.check) {
      // every expect: the drawn flange against the simulator's
      let worst = 0;
      const rows = s.expects.map((e) => {
        seek(e.t);
        const st = controls.state;
        const f = st.arms[e.arm] ? st.arms[e.arm].flange : st.parts[e.arm];
        if (!f) return { arm: e.arm, t: e.t, want: e.xyz, got: null, off: Infinity };
        const d = Math.hypot(f[0] - e.xyz[0], f[1] - e.xyz[1], f[2] - e.xyz[2]);
        worst = Math.max(worst, d);
        return { arm: e.arm, t: e.t, want: e.xyz, got: f, off: +d.toFixed(4) };
      });
      seek(options.time !== undefined ? options.time : 0);
      const out = document.getElementById('check') || document.body.appendChild(Object.assign(document.createElement('pre'), { id: 'check' }));
      const off = rows.filter((r) => r.off > 0.01).sort((a, b) => b.off - a.off).slice(0, 8);
      out.textContent = JSON.stringify({ off, expects: rows.length, worst: worst === Infinity ? 1e9 : +worst.toFixed(4), ghosts: s.ghosts.length, steps: s.steps.length, duration, arms: Object.keys(arms), fixtures: Object.keys(fixtures), parts: Object.keys(parts), state: controls.state });
    }
    document.title = document.title.replace(/: (ready|error)$/, '') + ': ready';
  }).catch((e) => {
    const err = document.getElementById('err');
    if (err) err.textContent = String(e.message || e);
    document.title = document.title.replace(/: (ready|error)$/, '') + ': error';
  });
  controls.ready = ready;
  controls.stopAll = () => { stopper.abort(); running = false; controls.stop(); ui.remove(); renderer.dispose(); host.removeChild(renderer.domElement); };
  window.cell = controls;
  return controls;
}
