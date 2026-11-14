// The page's controls round the player: riding the flight, flying,
// walking; showing a scene in place of the one before; and, when the
// page can build scenes (X_eTaL linked in as WebAssembly), a seed.
// ?t=SECONDS holds the flight at a moment (a still, for a test or a
// recording); ?hud=0 hides the controls; ?mode=fly or walk starts
// there; ?seed=N builds that city first.
import { play } from './player.js';

export function start() {
  const q = new URLSearchParams(location.search);
  const view = document.getElementById('view');
  const stat = document.getElementById('stat');
  const err = document.getElementById('err');
  const buttons = ['ride', 'fly', 'walk'].map((m) => document.getElementById(m));
  if (q.get('hud') === '0') document.getElementById('hud').style.display = 'none';
  if (q.has('about')) document.getElementById('about').open = true;
  const mark = (mode) => buttons.forEach((b) => b.classList.toggle('on', b.id === mode));
  let city = null, mode = q.get('mode') || 'ride', built = '';
  const live = {
    show(calls) {
      if (city) city.stop();
      err.textContent = '';
      const options = {
        mode, keep: q.has('t'),
        onMode: (m) => { mode = m; mark(m); },
        onFrame: (f) => { stat.textContent = `${f.mode}  ${f.rate.toFixed(0)} frames a second  at ${f.eye.x.toFixed(0)} ${f.eye.y.toFixed(0)} ${f.eye.z.toFixed(0)}${built}`; },
      };
      if (q.has('t')) options.time = Number(q.get('t'));
      city = play(view, calls, options);
      mark(mode);
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
  buttons.forEach((b) => b.addEventListener('click', () => { if (city) city.mode(b.id); }));
  globalThis.xetalThree = { show: live.show };
  return live;
}
