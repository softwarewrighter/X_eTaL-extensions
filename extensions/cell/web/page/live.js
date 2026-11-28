// The assembly cell, live: the page's side. The simulator runs here as
// WebAssembly (w: window.wasmBindings, the session of src/lib.rs);
// every control writes a command in the cell's command language and
// runs it exactly as the command box does, so what a button does is
// what typing it does. After each command the cell's timeline is shown
// again by the cell player, playing what the command did.
//   ?program=NAME runs that program first (solder, pick, arms; solder when not
//   given, so the list of steps is there to step through; ?program=
//   none starts with the demo tray and nothing done); ?t=SECONDS
//   holds that moment; ?check has the player measure its drawing
//   against the simulator's numbers (#check, for headless Chrome);
//   ?hud=0 hides the controls.
import { cellPlayer } from '../static/cell.js';

const q = new URLSearchParams(location.search);
const $ = (id) => document.getElementById(id);

// ?mcp: the page follows an MCP session (xetal-cell-mcp serves it):
// the cell's commands, scene and state are the server's, asked over
// HTTP; what an agent does is played as it comes, and the page's own
// controls drive the same cell. Everything else (the programs, X_eTaL)
// stays the page's own.
function remote(w) {
  const ask = (method, path, body) => {
    const x = new XMLHttpRequest();
    x.open(method, path, false);
    x.send(body === undefined ? null : body);
    return x.responseText;
  };
  // a module's exports are read-only: a plain copy, the session's calls over them
  return Object.assign({ ...w }, {
    command: (text) => ask('POST', '/mcp/command', text),
    reset: (circuit) => ask('POST', '/mcp/reset?circuit=' + encodeURIComponent(circuit)),
    clear: () => {},
    scene: () => ask('GET', '/mcp/scene'),
    state: () => ask('GET', '/mcp/state'),
    version: () => JSON.parse(ask('GET', '/mcp/version')).version,
    last: () => JSON.parse(ask('GET', '/mcp/last')),
  });
}

export function start(local) {
  const w = q.has('mcp') ? remote(local) : local;
  let player = null;
  let seenOwn = -1;
  const view = $('view');
  // show the cell's timeline, playing from `from` to `to` (or holding t)
  const show = (from, to, calls) => {
    // the detailed programs open close up on the work (the whole cell
    // is a meter across; a lead is half a millimeter)
    const detailed = ['solder', 'solder-stations', 'weld', 'weld-stations', 'leads', 'wire', 'support', 'turn'].includes(q.get('program') || 'solder');
    const options = { models: './models/', hud: q.get('hud') !== '0', check: q.has('check'), close: detailed && !q.has('view'), scopeLeft: 360 };
    // the bending rig stands beside the turntable: open on it
    if ((q.get('program') || 'solder') === 'bend-rig' && !q.has('view') && !q.has('look')) options.look = { at: [-55, 300, -100], dist: 150 };
    if (player) { options.view = player.view; player.stopAll(); }
    if (q.has('t')) options.time = Number(q.get('t'));
    else if (from !== undefined) options.time = from;
    player = cellPlayer(view, calls || w.scene('live'), options);
    if (!q.has('t') && to !== undefined && to > from) player.ready.then(() => player.play(from, to));
  };
  const log = (o, text) => {
    const li = document.createElement('li');
    if (o.error) {
      li.className = 'refused';
      li.textContent = `${text}: ${o.error}`;
    } else {
      li.className = o.ok ? (o.dry ? 'dry' : 'ok') : (o.refused ? 'refused' : 'stopped');
      const why = o.refused ? `refused: ${o.refused}` : o.stopped ? `stopped: ${o.stopped}` : `${o.t0.toFixed(2)} to ${o.t1.toFixed(2)} s`;
      const notes = [...(o.events || []), ...(o.warnings || []).map((x) => 'warning: ' + x)];
      li.innerHTML = '';
      const head = document.createElement('div');
      head.textContent = `${o.dry ? 'dry: ' : ''}${o.command} -- ${why}`;
      li.append(head);
      if (notes.length) { const n = document.createElement('div'); n.className = 'notes'; n.textContent = notes.join('; '); li.append(n); }
    }
    $('log').prepend(li);
  };
  // run commands on the cell, log them, play what they did
  const run = (text) => {
    const r = JSON.parse(w.command(text));
    if (w.version) seenOwn = w.version();
    if (r.error) { log(r, text); return r; }
    r.outcomes.forEach((o) => log(o));
    if (r.outcomes.some((o) => !o.dry)) show(r.t0, r.t1);
    refresh();
    return r;
  };
  const arm = () => $('arm').value;
  const tip = () => JSON.parse(w.state()).arms.find((a) => a.name === arm()).tip;
  const fill = (select, items) => {
    const keep = select.value;
    select.innerHTML = '';
    for (const [value, label] of items) select.add(new Option(label, value));
    if ([...select.options].some((o) => o.value === keep)) select.value = keep;
  };
  // the controls follow the cell: parts, their grasps, what each arm holds
  const refresh = () => {
    const s = JSON.parse(w.state());
    fill($('part'), s.parts.filter((p) => !p.lost).map((p) => [p.id, `${p.id} (${p.kind})${p.holder ? ', held by ' + p.holder : ''}`]));
    const p = s.parts.find((x) => x.id === $('part').value);
    fill($('grasp'), p ? p.grasps.map((g) => [g, g]) : []);
    const a = s.arms.find((x) => x.name === arm());
    $('holds').textContent = `after the last command, ${arm()} has ${a.tool ? 'the ' + a.tool : 'no tool'}${a.holding ? ', holding ' + a.holding : ''}, its tip at ${a.tip.slice(0, 3).map((x) => x.toFixed(1)).join(' ')}; the next command starts there`;
    $('vise').textContent = s.vise === 'closed' ? 'Open the vise' : 'Close the vise';
  };
  const n = (id) => Number($(id).value) || 0;
  const on = (id, f) => $(id).addEventListener('click', f);
  on('reset', () => { const r = JSON.parse(w.reset($('circuit').value)); $('log').innerHTML = ''; r.outcomes.forEach((o) => log(o)); show(0, 0); refresh(); });
  // a program run in a worker, off the page's thread, a command at a
  // time: the page draws and answers while it runs (the blinker: hundreds
  // of commands), what is being simulated shown beside the button; the
  // cell handed back at the end. Where there is no worker (an MCP
  // session's cell is on the server), a command at a time here, the page
  // yielding between them.
  const busy = document.createElement('span');
  busy.id = 'busy';
  busy.style.cssText = 'margin-left:8px;font-size:12px;opacity:.85';
  $('run-program').after(busy);
  const spin = ['|', '/', '-', '\\'];
  let worker = null;
  const inWorker = (lines, onStep) => new Promise((done, fail) => {
    if (!worker) worker = new Worker(new URL('./cell-worker.js', import.meta.url), { type: 'module' });
    worker.onerror = (e) => fail(e.message || 'the worker failed');
    worker.onmessage = ({ data }) => {
      if (data.error) fail(data.error);
      else if (data.done) done(data.world);
      else onStep(data);
    };
    worker.postMessage({ lines, world: w.world() });
  });
  const here = async (lines, onStep) => {
    for (let k = 0; k < lines.length; k++) {
      onStep({ k, n: lines.length, line: lines[k] });
      await new Promise((go) => setTimeout(go, 0));
      const r = JSON.parse(w.command(lines[k]));
      onStep({ k, r });
      if (r.error || r.outcomes.some((o) => !o.ok)) break;
    }
  };
  on('run-program', async () => {
    const button = $('run-program');
    if (button.disabled) return;
    // a program that begins with its own reset starts the cell over
    const text = w.program($('program').value);
    if (/^\s*reset\b/m.test(text)) { w.clear(); $('log').innerHTML = ''; }
    const lines = text.split('\n').filter((l) => l.trim() && !l.trim().startsWith('#'));
    button.disabled = true;
    const onStep = ({ k, n, line, r }) => {
      if (line !== undefined) { busy.textContent = `${spin[k % 4]} simulating ${k + 1} of ${n}: ${line.trim().slice(0, 48)}`; return; }
      if (r.error) log(r, lines[k]);
      else r.outcomes.forEach((o) => log(o));
    };
    try {
      if (w.world && typeof Worker === 'function') {
        const world = await inWorker(lines, onStep);
        const wrong = w.set_world(world);
        if (wrong) throw new Error(wrong);
      } else {
        await here(lines, onStep);
      }
    } catch (e) {
      log({ error: String(e.message || e) }, 'run');
    } finally {
      if (w.version) seenOwn = w.version();
      busy.textContent = '';
      button.disabled = false;
      refresh();
      show(0, JSON.parse(w.state()).t);
    }
  });
  // the bands in the microscope's picture, read from its pixels by the
  // simulator's own decoder (marks.rs): what a sensed reading sees
  on('read-picture', () => {
    if (player && player.software) { $('reading').textContent = 'reading the microscope\'s picture needs WebGL, which this browser does not have here'; return; }
    const seen = player ? player.readPicture() : [];
    const kinds = {};
    for (const p of Object.values(JSON.parse(w.state()).parts || {})) if (p && p.id) kinds[p.id] = p.kind;
    const parts = JSON.parse(w.state()).parts;
    const kindOf = (id) => (Array.isArray(parts) ? (parts.find((p) => p.id === id) || {}).kind : (parts[id] || {}).kind) || kinds[id];
    const doubt = (c) => (c >= 0.6 ? '' : ' -- doubtful, zoom in or turn it');
    $('reading').textContent = seen.length
      ? seen.map((x) => {
        if (x.bands) {
          const r = JSON.parse(local.read_bands(JSON.stringify(x)));
          const raw = q.has('raw') ? ` [samples ${JSON.stringify(x.bands.map((c) => c.map(Math.round)))} body ${JSON.stringify(x.body.map(Math.round))} true ${JSON.stringify(x.body_true)}]` : '';
          return `${x.id}: ${r.bands.join(' ')} = ${r.value || r.error} (confidence ${r.confidence.toFixed(2)})${doubt(r.confidence)}${raw}`;
        }
        if (!x.text) return `${x.id}: unreadable, ${x.why}`;
        const r = JSON.parse(local.decode_mark(kindOf(x.id) || '', x.text));
        return `${x.id}: "${x.text}" = ${r.value || r.error} (confidence ${x.confidence.toFixed(2)})${doubt(x.confidence)}`;
      }).join('; ')
      : 'no marked part in the microscope\'s picture: aim it at one (identify R1)';
  });
  // ?read: the picture read once the page has drawn (for checking it headlessly)
  if (q.has('read')) setTimeout(() => $('read-picture').click(), 4000);
  on('take', () => run(`tool ${arm()} ${$('tool').value}`));
  on('pick', () => run(`pick ${arm()} ${$('part').value} grasp=${$('grasp').value}`));
  on('release', () => { const h = JSON.parse(w.state()).arms.find((a) => a.name === arm()).holding; run(h ? `release ${arm()} ${h}` : `grip ${arm()} open`); });
  on('place', () => { const h = JSON.parse(w.state()).arms.find((a) => a.name === arm()).holding || $('part').value; run(`place ${arm()} ${h} at=${n('px')},${n('py')},${n('pz')}`); });
  on('home', () => run(`joints ${arm()} 0 0 0 0 90 0`));
  for (const [id, d] of [['xm', [-1, 0, 0]], ['xp', [1, 0, 0]], ['ym', [0, -1, 0]], ['yp', [0, 1, 0]], ['zm', [0, 0, -1]], ['zp', [0, 0, 1]]]) {
    on(id, () => { const t = tip(), s = n('step'); run(`move ${arm()} to=${(t[0] + d[0] * s).toFixed(3)},${(t[1] + d[1] * s).toFixed(3)},${(t[2] + d[2] * s).toFixed(3)}`); });
  }
  on('turn-l', () => run('stage turn=90'));
  on('turn-r', () => run('stage turn=-90'));
  on('vise', () => run(JSON.parse(w.state()).vise === 'closed' ? 'vise open' : 'vise close'));
  on('rewind', () => run(`rewind t=${player ? player.state.t.toFixed(6) : 0}`));
  $('box').addEventListener('submit', (e) => {
    e.preventDefault();
    let text = $('text').value.trim();
    if (!text) return;
    if ($('dry').checked) text = text.split('\n').map((l) => (l.trim() && !l.trim().startsWith('#') ? l + ' dry=1' : l)).join('\n');
    run(text);
  });
  $('arm').addEventListener('change', refresh);
  $('part').addEventListener('change', refresh);
  // X_eTaL in the page: a program's output, and its scene shown
  $('xetal-code').value = w.example();
  on('xetal-run', () => {
    const o = JSON.parse(w.xetal($('xetal-code').value));
    $('xetal-out').textContent = o.err ? o.err : `${o.out.split('\n').length - 1} lines from X_eTaL in ${o.millis.toFixed(0)} ms` + (o.out.startsWith('cell(') ? ' -- its scene is shown; Show the cell goes back' : '\n' + o.out);
    if (!o.err && o.out.startsWith('cell(')) show(0, undefined, o.out);
  });
  on('xetal-back', () => show(0, 0));
  $('facts').innerHTML = w.about();
  on('open-about', () => $('about').showModal());
  if (q.get('hud') === '0') $('panel').style.display = 'none';
  // following an MCP session: the cell is the server's as it stands;
  // each change an agent makes is played as it comes
  if (q.has('mcp')) {
    w.install();
    $('hint').textContent = 'Following an MCP session: what an agent does here is played as it comes; these controls drive the same cell.';
    let seen = w.version();
    const last = w.last();
    show(last && last.t0 !== undefined ? last.t0 : 0, last && last.t1);
    refresh();
    setInterval(() => {
      const v = w.version();
      if (v === seen) return;
      seen = v;
      // this page's own command: already shown
      if (v === seenOwn) return;
      const r = w.last();
      (r.outcomes || []).forEach((o) => log(o));
      show(r.t0, r.t1);
      refresh();
    }, 500);
    return { run, show };
  }
  // the first cell: a program asked for, else the demo tray
  w.install();
  // a program that begins with its own reset runs on an empty cell
  const name = q.has('program') ? q.get('program') : 'solder';
  const asked = name === 'none' ? '' : w.program(name);
  if (/^\s*reset\b/m.test(asked)) w.clear(); else JSON.parse(w.reset(q.get('circuit') || 'demo'));
  if (asked) {
    $('program').value = name;
    // the hint says which program is loaded
    const what = {
      weld: 'a disc welded to a disc in the vise, then an LED, its lead clipped to ground first: the three welded into one in the vise',
      'solder-stations': 'the bench, with a lap joint: the chip in the vise legs up; the rig bends the resistor\'s lead and shears it to a 4 mm end, the arm lays that end alongside pin 3\'s leg, a pincer on the turntable grips it so the arm can go, the turntable presents the joint to the iron station, which feeds its own solder -- good, a 3.6 mm lap',
      'weld-stations': 'the stage brings a disc lead to the welder station, which comes up from below; an arm lays a second disc lead on it, and it is welded',
      pick: 'a chip picked from the tray and put in the vise',
      plan: 'written by the planner (CellPlan.xtl, in X_eTaL) from a netlist -- four resistors dead-bug on a DIP-8 with lap joints: the chip into the vise legs up, then each resistor\'s lead bent 90 degrees in the rig and sheared to a 4 mm end, laid alongside its pin\'s leg (fanned out so bodies keep apart), pinched by the pincer on its side, soldered and let go; one side of the chip, then the other; no shorts, every joint a good 3.6 mm lap',
      multivibrator: 'a two-transistor astable multivibrator, in progress, built Manhattan style on a ground-plane tile: the tile in the vise, its ten posts up (two on the plane, eight on pads, a net each); VCC across the far end; the four resistors each laid onto its post from outside its row and jumped to VCC; the transistors\' collectors onto their posts, the emitters and bases bent out and jumped; the capacitors and the LEDs\' anodes between neighboring posts -- check: 4 of 8 nets; the LEDs\' cathodes and the battery are not joined yet',
      wrap: 'wrap joints: the chip in the vise legs up; the wire feeder feeds a jumper from pin 4 across the chip\'s end to pin 5, each end passing its leg just below the tip and on past it, long enough for 1.75 turns and a tail; arm A takes the wrap tool, and at each pin its bit slips over the leg\'s end, winds the wire round it and holds it while the iron solders, then its blade cuts the tail and the bit lifts off -- inspect: good, a wrap of 1.75 turns (1.5 or more holds before the solder flows; fewer is weak; the tool winds under two)',
      lap: 'a lap joint: the chip in the vise legs up (dead-bug); the rig bends the resistor\'s lead 90 degrees 12 mm from its body and shears it, and the arm lays the 4 mm end alongside pin 3, touching it and pointing down the leg -- two parallel pieces of metal soldered for 1/8 inch hold far better than a lead\'s end against a leg (that butt joint is weak); inspect: good, a 3.6 mm lap',
      blinker: 'the 555 LED blinker, the whole circuit, planned by search from its netlist (CellPlan.xtl, each step tried on the simulator), dead-bug with lap joints: the chip legs up in the vise; enameled jumpers between legs first (pin 4 to 8, then 2 to 6 over it); a lead of R1, R2, R3, C1 and C2 each laid alongside its pin\'s leg, the resistors\' leads bent in the rig, the capacitors\' straight leads lowered from above; R1, R2 and R3 trimmed at the cutter; R3\'s end bent up by a pincer and the LED lapped onto it; jumpers with lapped ends for the rest; the battery snap set on the turntable and jumped to pins 8 and 1 -- check: 7 of 7 nets, every joint a good lap, no shorts',
      enamel: 'enameled (magnet) wire: the rig bends the wire\'s end and the arm lays it alongside pin 3\'s leg, the wire run over pin 4, clear of it; the iron at 400 C burns the enamel off only where it heats the wire, at the joint -- good, a lap; shorts: none',
      insulate: 'heat-shrink sleeving: the resistor\'s lead will run along the chip\'s pin row to pin 3, past pins 2 and 1; first sleeve station S6 slides a blue sleeve onto it and shrinks it with hot air; then the rig bends the lead and the arm lays its end alongside pin 3\'s leg -- the bent end longer than the lap, the sleeved lead passes over pins 2 and 1, clear of them by 0.7 mm; soldered, a good lap',
      tester: 'the component tester: arm A holds each part in free air over the turntable while the probes (pincers P1 and P2) touch each pair of its leads; it says what the part is from the readings alone, then verify checks it against the netlist and its marking -- R2 is mislabeled, R3 the wrong part, C2 open, D2\'s band reversed. The readings are in the step list\'s events',
      'tester-b': 'the component tester on a PNP transistor, a shorted capacitor and a good electrolytic',
      identify: 'the microscope reads the parts\' markings: the resistor\'s bands and the LED\'s color in the tray; the capacitor\'s printed face is sideways and cannot be read, so it goes in the vise and the positioner tilts it toward the scope -- at 30 degrees the code may be misread (a warning), at 45 it reads 104, 100 nF. The readings are in the step list\'s events',
      'bend-rig': 'the bending rig: the resistor\'s lead 2 slid in 3 mm from the body, the clamp and anvil close on it and the former bends it down round the mandrel, the shear cuts it to 8 mm; then lead 1 bent 12 mm from the body and sheared to a 4 mm end, laid alongside pin 3\'s leg and soldered, a lap joint',
      support: 'a steel support is welded to a part and held by a pincer; the vise opens and the support alone holds the part while the turntable turns and tilts it; then the support is cut away',
      wire: 'the wire feeder feeds 40 mm of wire, the arm takes it and the feeder cuts it; the rig bends its end, which the arm lays alongside pin 3\'s leg and the iron station solders (a lap joint); a pincer bends its other half down',
      turn: 'the arm takes the chip out of the vise and puts it back on its side, its pins pointing across; the turntable turns and tilts it',
      leads: 'a resistor\'s lead bent in the rig and laid alongside pin 3\'s leg, soldered at the bench (a lap joint); then its other lead bent down round a mandrel by a pincer and trimmed to 8 mm by the cutter station',
      arms: 'each arm taking a tool and moving',
    }[name];
    if (what) $('hint').innerHTML = $('hint').innerHTML.replace(/The solder program is loaded: [^.]*\./, `The ${name} program is loaded: ${what}.`);
    if (/^\s*reset circuit=(\w+)/m.test(asked)) $('circuit').value = asked.match(/^\s*reset circuit=(\w+)/m)[1];
    const r = JSON.parse(w.command(asked));
    if (r.outcomes) r.outcomes.forEach((o) => log(o));
    show(q.has('t') ? undefined : 0, undefined);
  } else {
    show(0, 0);
  }
  refresh();
  return { run, show };
}
