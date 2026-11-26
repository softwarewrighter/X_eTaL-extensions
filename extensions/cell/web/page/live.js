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
    const options = { models: './models/', hud: q.get('hud') !== '0', check: q.has('check') };
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
  on('run-program', () => {
    // a program that begins with its own reset starts the cell over
    const text = w.program($('program').value);
    if (/^\s*reset\b/m.test(text)) { w.clear(); $('log').innerHTML = ''; }
    run(text);
    show(0, JSON.parse(w.state()).t);
  });
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
      'solder-stations': 'the bench: an arm brings a resistor lead to a chip pin, a pincer on the turntable grips it so the arm can go, the turntable presents the joint to the iron station, which feeds its own solder',
      'weld-stations': 'the stage brings a disc lead to the welder station, which comes up from below; an arm lays a second disc lead on it, and it is welded',
      pick: 'a chip picked from the tray and put in the vise',
      support: 'a steel support is welded to a part and held by a pincer; the vise opens and the support alone holds the part while the turntable turns and tilts it; then the support is cut away',
      wire: 'the wire feeder feeds 40 mm of wire, the arm takes it and the feeder cuts it, its end is soldered to a chip pin, and a pincer bends its other half down',
      turn: 'the arm takes the chip out of the vise and puts it back on its side, its pins pointing across; the turntable turns and tilts it',
      leads: 'a resistor soldered to a chip pin at the bench, then its other lead bent down round a mandrel by a pincer and trimmed to 8 mm by the cutter station',
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
