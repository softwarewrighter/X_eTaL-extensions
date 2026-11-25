// A camera that walks: a view with perspective from where you stand, in
// place of camera. A module of the board player (../board.js says what
// a module is).
//   eye(nums('x z y', 3)[0], nums('hx hz pitch', 3)[0], lens)
//   eye(nums('x z y', 3)[0], nums('hx hz pitch', 3)[0], lens, dt)
// place: where the eye is, x east, z south, y up, in tiles (y is the
//   eye's own height above the ground's zero: the program adds what it
//   stands on).
// way: the heading along x and z (any length; 0 -1 is north) and the
//   pitch in degrees, up from level (90 straight up, -90 straight down).
//   A heading of 0 0 keeps the way the eye faced (north at first).
// lens: the field of view from the picture's bottom to its top, in
//   degrees (60 is a walker's view). Degrees and not the millimeters of
//   the city's fly: a board's picture is fitted to whatever shape its
//   element has, and a number of degrees says what is seen whatever that
//   is and names no film.
// dt: the seconds the eye takes to come here from where it was (a
//   tick's length, as piece has). Without it, the time between this
//   scene and the one before (at most 0.25 s).
// It is an each call: a scene with an eye line is taken with this
// camera (a camera line after it gives the picture back to the board's
// own; the later of the two wins), and each scene shown moves the eye.
// Between ticks it glides: from where it is drawn when the line comes
// to where the line puts it, in a straight line, the heading turning the
// short way round, over dt. The first scene with an eye (and the first
// after scenes without one) puts it there at once.
// Held (options.time, a page's ?t=): the eye is options.time seconds
// into the glide from where the scene before put it; with no dt it is
// where the line says, since a measured time is not the same twice.
//
// The keys and the pointer (options.onInput): the program moves the
// eye and the player only looks, so nothing here moves anything. What
// is held and how far the pointer went are handed to the page:
//   onInput({ keys, pressed, released, look, locked, eye, seconds })
//     keys      what is held now: KeyboardEvent.code values ('KeyW',
//               'ArrowLeft', 'ShiftLeft', 'Space'), in the order they
//               went down, and 'Mouse0' 'Mouse1' 'Mouse2' for the
//               pointer's buttons while it is locked
//     pressed   what went down since the report before (a tap shorter
//               than a report is here and not in keys)
//     released  what came up since the report before
//     look      [dx, dy]: how far the pointer moved since the report
//               before, in CSS pixels, right and down positive; only
//               while it is locked (the city turns 0.0022 radians a
//               pixel)
//     locked    whether the pointer is locked to the picture
//     eye       whether the picture is the eye's (else the board's own
//               camera's)
//     seconds   the time since the report before (a report's length,
//               1 / inputRate, for the first after a quiet time)
// called options.inputRate times a second (20 unless given) while
// anything is held, pressed, released or moved, and not otherwise; the
// report that says a key came up is the last until something happens.
// A page that ticks at another rate adds the looks up and keeps the
// last keys. Keys typed into a form's field, typed while a dialog is
// open, or held with Ctrl, Alt or the command key are not reported;
// when the window loses the keys, everything held is released.
// With onInput and an eye, a click on the picture locks the pointer to
// it (Esc frees it), as the city's player does to fly or walk; that
// click is not reported. Without onInput there are no listeners at all.
//
// A click: onPoint's numbers are the board's own camera's (a place in
// the view's units that the program's map turns into a tile), and mean
// nothing through a lens, so while the picture is the eye's onPoint is
// not called. A click with the pointer locked is 'Mouse0' in pressed:
// it is on the middle of the picture, along the way the program itself
// said the eye looks, so the program has the ray already.
//
// Near and far: 0.05 and 400 tiles (a wall can be stood against; a
// board a few dozen tiles across and what is round it are inside, and
// 24 bits of depth tell 3 mm apart at 50 tiles).
const NEAR = 0.05, FAR = 400, LONGEST = 0.25;
// the keys a page would otherwise scroll or act on (the city's moving keys)
const MOVING = ['KeyW', 'KeyA', 'KeyS', 'KeyD', 'KeyQ', 'KeyE', 'Space', 'ArrowUp', 'ArrowDown', 'ArrowLeft', 'ArrowRight'];

export default function eye(ctx) {
  const { THREE, options } = ctx;
  const cam = new THREE.PerspectiveCamera(60, 16 / 9, NEAR, FAR);
  cam.rotation.order = 'YXZ';
  const held = options.time !== undefined;
  // x, y, z as Three.js has them (y up), the turn about y, the pitch (radians), the lens
  const from = [0, 0, 0, 0, 0, 60], to = [0, 0, 0, 0, 0, 60], at = [0, 0, 0, 0, 0, 60];
  let seen = false, was = false, since = 0, gap = 0, lens = 0;
  const active = () => ctx.camera === cam;

  // where the eye is now: along the glide
  const place = (now) => {
    const passed = held ? options.time : (now - since) / 1000;
    const u = gap > 0 ? Math.min(1, Math.max(0, passed / gap)) : 1;
    for (let i = 0; i < 6; i++) at[i] = from[i] + (to[i] - from[i]) * u;
  };
  const aim = () => {
    cam.position.set(at[0], at[1], at[2]);
    cam.rotation.set(at[4], at[3], 0);
    if (at[5] !== lens) { lens = at[5]; cam.fov = lens; cam.updateProjectionMatrix(); }
  };

  const call = (where, way, degrees, dt) => {
    const now = performance.now();
    // a second eye line of one scene only says again where to
    if (!seen) {
      if (held) from.splice(0, 6, ...to); else { place(now); from.splice(0, 6, ...at); }
      gap = dt !== undefined ? Math.max(0, dt) : held ? 0 : Math.min(LONGEST, (now - since) / 1000);
      since = now;
    }
    const turn = way[0] || way[1] ? Math.atan2(-way[0], -way[1]) : to[3];
    to.splice(0, 6, where[0], where[2], where[1], turn, way[2] * Math.PI / 180, degrees);
    if (!was) { from.splice(0, 6, ...to); gap = 0; }
    // the short way round
    const round = to[3] - from[3];
    to[3] = from[3] + round - 2 * Math.PI * Math.round(round / (2 * Math.PI));
    seen = true;
    ctx.camera = cam;
    place(now);
    aim();
  };

  // the keys and the pointer, handed to the page
  const el = ctx.renderer.domElement;
  const keys = new Set(), pressed = [], released = [], look = [0, 0];
  let timer = 0, last = 0;
  const report = () => {
    const now = performance.now();
    if (!keys.size && !pressed.length && !released.length && !look[0] && !look[1]) { last = 0; return; }
    const seconds = last ? (now - last) / 1000 : 1 / (options.inputRate || 20);
    last = now;
    const input = { keys: [...keys], pressed: [...pressed], released: [...released], look: [look[0], look[1]], locked: document.pointerLockElement === el, eye: active(), seconds };
    pressed.length = 0; released.length = 0; look[0] = 0; look[1] = 0;
    options.onInput(input);
  };
  const down = (code) => { if (!keys.has(code)) { keys.add(code); pressed.push(code); } };
  const up = (code) => { if (keys.delete(code)) released.push(code); };
  if (options.onInput) {
    el.tabIndex = 0;
    el.style.outline = 'none';
    el.addEventListener('click', () => {
      if (!active()) return;
      el.focus();
      if (document.pointerLockElement !== el && el.requestPointerLock) el.requestPointerLock();
    }, ctx.until);
    document.addEventListener('mousemove', (e) => {
      if (document.pointerLockElement !== el || !active()) return;
      look[0] += e.movementX;
      look[1] += e.movementY;
    }, ctx.until);
    // a button, once the pointer is locked (the click that locks it is not one)
    el.addEventListener('mousedown', (e) => { if (document.pointerLockElement === el && active()) down(`Mouse${e.button}`); }, ctx.until);
    window.addEventListener('mouseup', (e) => up(`Mouse${e.button}`), ctx.until);
    document.addEventListener('pointerlockchange', () => {
      if (document.pointerLockElement !== el) for (const k of [...keys]) if (k.startsWith('Mouse')) up(k);
    }, ctx.until);
    window.addEventListener('keydown', (e) => {
      if ((e.target && /INPUT|TEXTAREA|SELECT/.test(e.target.tagName)) || document.querySelector('dialog[open]')) return;
      if (e.ctrlKey || e.altKey || e.metaKey) return;
      down(e.code);
      if (MOVING.includes(e.code)) e.preventDefault();
    }, ctx.until);
    window.addEventListener('keyup', (e) => up(e.code), ctx.until);
    window.addEventListener('blur', () => { for (const k of [...keys]) up(k); }, ctx.until);
    timer = setInterval(report, 1000 / (options.inputRate || 20));
  }
  // a click through a lens is not a place in the view's units (board.js
  // reads options.onPoint at each click, so it is this it calls)
  const point = options.onPoint;
  if (point) options.onPoint = (x, y, e) => { if (!active()) point(x, y, e); };

  return {
    each: { eye: call },
    begin() { was = seen; seen = false; },
    frame() {
      if (!active()) return;
      place(performance.now());
      aim();
    },
    size(wide, high) {
      cam.aspect = wide / high;
      cam.updateProjectionMatrix();
    },
    // where the eye is drawn (x z y), the way it looks (hx hz pitch) and its lens
    count() {
      if (!active()) return {};
      return { eye: [at[0], at[2], at[1]], way: [-Math.sin(at[3]), -Math.cos(at[3]), at[4] * 180 / Math.PI], lens: at[5] };
    },
    stop() {
      if (timer) clearInterval(timer);
      if (point) options.onPoint = point;
      if (document.pointerLockElement === el) document.exitPointerLock();
    },
  };
}
