// The page's controls round the players: riding the flight, flying,
// walking; showing a scene in place of the one before; and, when the
// page can build scenes (X_eTaL linked in as WebAssembly), a seed. A
// scene whose first call is view is a game's board (board.js): no
// flight, and shown again it changes in place.
// ?t=SECONDS holds the flight at a moment (a still, for a test or a
// recording); ?hud=0 hides the controls; ?mode=fly or walk starts
// there; ?seed=N builds that city first. For a board, ?t=SECONDS holds
// every piece that long after its tick.
// setup: media (the city's pictures' folder), models (the folder of a
// board's .glb pieces), textures (the folder of a board's ground textures), onPoint(x, y, event) (a click on a board, in
// its view's units), onInput(input) (a board's keys and pointer, for the
// program that moves its eye: board/eye.js says what input holds; with
// it, a board seen through an eye says how to look round).
import { play } from './player.js';
import { board, isBoard } from './board.js';

export function start(setup = {}) {
  const q = new URLSearchParams(location.search);
  const view = document.getElementById('view');
  const stat = document.getElementById('stat');
  const err = document.getElementById('err');
  const buttons = ['ride', 'fly', 'walk'].map((m) => document.getElementById(m));
  if (q.get('hud') === '0') document.getElementById('hud').style.display = 'none';
  // About: a dialog where the page has one (else a section that opens)
  const about = document.getElementById('about'), opener = document.getElementById('open-about');
  if (opener) opener.addEventListener('click', () => about.showModal());
  if (q.has('about')) { if (about.showModal) about.showModal(); else about.open = true; }
  const mark = (mode) => buttons.forEach((b) => b.classList.toggle('on', b.id === mode));
  let city = null, mode = q.get('mode') || 'ride', built = '', high = 0;
  const live = {
    show(calls) {
      if (isBoard(calls)) { live.board(calls); return; }
      if (city) city.stop();
      document.body.classList.remove('board', 'eye');
      err.textContent = '';
      const options = {
        mode, keep: q.has('t'), media: setup.media,
        onMode: (m) => { mode = m; mark(m); },
        onFrame: (f) => {
          // ?bench: the rate in the title, for a measurement from outside
          if (q.has('bench')) { high = f.time > 3 ? Math.max(high, f.slowest) : 0; document.title = `bench ${f.rate.toFixed(1)} a second, slowest frame ${high.toFixed(0)} ms`; }
          stat.textContent = `${f.mode}  ${f.rate.toFixed(0)} frames a second  at ${f.eye.x.toFixed(0)} ${f.eye.y.toFixed(0)} ${f.eye.z.toFixed(0)}${built}`;
        },
      };
      if (q.has('t')) options.time = Number(q.get('t'));
      city = play(view, calls, options);
      mark(mode);
      document.title = document.title.replace(/: (ready|error)$/, '') + ': ready';
    },
    // a board: made once, then each scene changes it in place
    board(calls) {
      err.textContent = '';
      if (city && city.board) { city.show(calls); return; }
      if (city) city.stop();
      document.body.classList.add('board');
      const options = {
        keep: q.has('t'), models: setup.models, textures: setup.textures, onPoint: setup.onPoint, onInput: setup.onInput,
        onFrame: (f) => {
          // through an eye: where it stands, and (when the page hears the keys) how to look round
          document.body.classList.toggle('eye', !!f.eye && !!setup.onInput);
          const stands = f.eye ? `  at ${f.eye.map((v) => v.toFixed(1)).join(' ')}` : '';
          const skipped = f.unknown ? `  ${f.unknown} unknown calls skipped` : '';
          stat.textContent = `board  ${f.tiles} tiles  ${f.pieces} pieces  ${f.rate.toFixed(0)} frames a second${stands}${skipped}`;
        },
      };
      if (q.has('t')) options.time = Number(q.get('t'));
      city = board(view, calls, options);
      document.title = document.title.replace(/: (ready|error)$/, '') + ': ready';
    },
    fail(e) {
      err.textContent = String((e && e.message) || e);
      document.title = document.title.replace(/: (ready|error)$/, '') + ': error';
    },
    // build(seed) makes the scene of a seed and shows it (through
    // xetalThree.show); it returns what to say, or throws
    seeds(build, first) {
      const form = document.getElementById('seed'), number = document.getElementById('number');
      form.style.display = 'block';
      const make = (seed) => {
        number.value = seed;
        try {
          built = '  ' + build(seed);
        } catch (e) {
          live.fail(e);
        }
      };
      form.addEventListener('submit', (e) => { e.preventDefault(); make(Number(number.value) || 0); });
      document.getElementById('another').addEventListener('click', () => make(Math.floor(Math.random() * 100000)));
      make(q.has('seed') ? Number(q.get('seed')) : first);
    },
  };
  buttons.forEach((b) => b.addEventListener('click', () => { if (city && city.mode) city.mode(b.id); }));
  globalThis.xetalThree = { show: live.show };
  return live;
}
