// The page round a game. A game is an X_eTaL program kept running as
// WebAssembly (xetal-ext-shell's play and page modules; `game_page!`
// exports the functions this file is given). This file gives it events
// and shows what it draws, and decides nothing about a game: which
// keys, how long a tick, and the words of every event are the page's,
// given as setup.
//   - the clock: every `tick` seconds, the line setup.tickLine(tick)
//     says ('tick 0.05'). A late timer is caught up, `catchUp` ticks at
//     most at once and the rest forgotten, and a hidden tab gets no
//     ticks and owes none when it is shown again
//   - a key: the line setup.keyLine(event) says, if it says one
//   - a click: the line setup.clickLine(x, y, event) says, x and y in
//     the view's units -- from the board player's onPoint, or, on the
//     flat picture, through the SVG's own map (the same units when the
//     picture's viewBox is the view's size)
//   - after each event, the picture the program showed (`[]S_HOW`, an
//     SVG document). Its <metadata id="scene"> is the board's calls,
//     which board.js draws into setup.view; the picture itself goes
//     into setup.flat, which then has the class `solid` (the page's
//     stylesheet hides there what the board draws, say
//     `#flat.solid .flat { display: none }`). Where there is no WebGL,
//     or a picture has no scene, the flat picture is all there is: no
//     class `solid`, the board's element hidden
//   - what the program printed, and its end, to the page's functions
import { board } from './board.js';

// Can this browser draw the board?
export function webgl() {
  try { return !!document.createElement('canvas').getContext('webgl2'); } catch { return false; }
}

// The scene a picture carries (the text of its metadata, as written
// before it was escaped for XML), or ''.
export function sceneOf(svg) {
  const found = svg.match(/<metadata id="scene">([\s\S]*?)<\/metadata>/);
  if (!found) return '';
  if (!found[1].includes('&')) return found[1];
  const named = { lt: '<', gt: '>', quot: '"', apos: "'", amp: '&' };
  return found[1].replace(/&(lt|gt|quot|apos|amp);/g, (_, n) => named[n]);
}

// A keyLine for the usual case: `names` maps a KeyboardEvent's key to
// the name the game knows ({ ' ': 'space', ArrowUp: 'up' }); a key not
// in it, a held key repeating and a key with Ctrl, Alt or Meta are the
// browser's, not the game's. `line(name)` says the event.
export function named(names, line) {
  return (e) => (e.ctrlKey || e.metaKey || e.altKey || e.repeat || !Object.hasOwn(names, e.key) ? '' : line(names[e.key]));
}

// Plays a game. `wasm` holds the page's exports: start(seed),
// event(line) (true once the game is over), picture(), printed(),
// error(), stop(). `setup`:
//   view        the element the board is drawn into
//   flat        the element the picture (SVG) is put into
//   seed        what start() rolls from when given none (1)
//   tick        seconds a tick; none, or no tickLine: no clock (events
//               are then given by the controls' event(line))
//   tickLine    (seconds) => the tick's line
//   catchUp     the most ticks given at once after a late timer (4)
//   keyLine     (KeyboardEvent) => a key's line, or '' for a key that
//               is not the game's; none: no keys
//   keysOn      what hears the keys (window)
//   clickLine   (x, y, event) => a click's line, or ''; none: no clicks
//   solid       false: the flat picture only (else: where WebGL is)
//   board       options for board.js (models, keep, pixels, onFrame,
//               time); onPoint is this file's
//   onPicture   (svg, calls) after each picture shown
//   onPrinted   (text) what the program printed since the last event
//   onEvent     (line, milliseconds) after each event: what X_eTaL took
//   onOver      (error) once, when the program has ended; '' if cleanly
// Returns its controls: start(seed), event(line), stop(), close(),
// over(), solid() (is the board drawing?), player() (the board's
// controls, or null).
export function game(wasm, setup) {
  const { view, flat } = setup;
  const gone = new AbortController(), until = { signal: gone.signal };
  const tick = setup.tickLine && setup.tick > 0 ? setup.tick : 0;
  const most = setup.catchUp || 4;
  let can = setup.solid !== false && !!view && webgl();
  let player = null, drawn = false, timer = 0, last = 0, owed = 0, running = false;

  // The board draws these calls, or (it cannot) nothing does.
  function draw(calls) {
    try {
      if (player) player.show(calls);
      else player = board(view, calls, { ...(setup.board || {}), onPoint: point });
      return true;
    } catch (e) {
      // no board after all: the flat picture from now on
      console.error(e);
      if (player) player.stop();
      player = null;
      can = false;
      return false;
    }
  }

  // Show what the program drew and say what it printed since last time.
  function show() {
    const svg = wasm.picture();
    if (svg) {
      const calls = sceneOf(svg);
      drawn = can && !!calls && draw(calls);
      if (view) view.hidden = !drawn;
      if (flat) {
        flat.innerHTML = svg;
        flat.classList.toggle('solid', drawn);
      }
      if (setup.onPicture) setup.onPicture(svg, calls);
    }
    const text = wasm.printed();
    if (text && setup.onPrinted) setup.onPrinted(text);
  }

  // Give the program lines; when it has ended, stop the clock and say so.
  function give(lines) {
    if (!running) return;
    let over = false;
    for (const line of lines) {
      const before = performance.now();
      over = wasm.event(line);
      if (setup.onEvent) setup.onEvent(line, performance.now() - before);
      if (over) break;
    }
    show();
    if (over) end();
  }
  function event(line) { if (line) give([line]); }

  function end() {
    if (!running) return;
    running = false;
    clearInterval(timer);
    timer = 0;
    if (setup.onOver) setup.onOver(wasm.error());
  }

  // The clock: the ticks owed since it last looked, a few at most.
  function clock() {
    const now = performance.now();
    owed += (now - last) / 1000;
    last = now;
    if (document.hidden) { owed = 0; return; }
    const n = Math.min(most, Math.floor(owed / tick + 1e-6));
    owed = n < most ? owed - n * tick : 0;
    if (n > 0) give(Array(n).fill(setup.tickLine(tick)));
  }

  function point(x, y, e) { if (setup.clickLine) event(setup.clickLine(x, y, e)); }

  // A click on the flat picture, through the SVG's own map (the board
  // hears its own clicks when it is drawing).
  const stage = flat ? flat.parentElement || flat : null;
  if (stage && setup.clickLine) {
    stage.addEventListener('click', (e) => {
      const svg = drawn ? null : flat.querySelector('svg');
      const map = svg && svg.getScreenCTM();
      if (!map) return;
      const p = new DOMPoint(e.clientX, e.clientY).matrixTransform(map.inverse());
      point(p.x, p.y, e);
    }, until);
  }
  if (setup.keyLine) {
    (setup.keysOn || window).addEventListener('keydown', (e) => {
      if (!running) return;
      const line = setup.keyLine(e);
      if (!line) return;
      e.preventDefault();
      event(line);
    }, until);
  }

  // Start the program from its beginning, and the clock.
  function start(seed) {
    clearInterval(timer);
    timer = 0;
    wasm.start(seed === undefined ? (setup.seed === undefined ? 1 : setup.seed) : seed);
    running = true;
    show();
    if (wasm.over()) { end(); return; }
    if (tick) {
      last = performance.now();
      owed = 0;
      timer = setInterval(clock, tick * 1000);
    }
  }

  return {
    start,
    event,
    // Stop the program where it waits (the page is told as at any end).
    stop() { if (running) { wasm.stop(); end(); } },
    // Stop, and take the board and the listeners away.
    close() {
      if (running) { wasm.stop(); running = false; }
      clearInterval(timer);
      gone.abort();
      if (player) player.stop();
      player = null;
    },
    over: () => !running,
    solid: () => drawn,
    player: () => player,
  };
}
