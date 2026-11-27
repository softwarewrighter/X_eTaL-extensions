// The cell's simulator in a worker: a program run off the page's thread,
// so the page draws and answers while it runs. It starts from the
// page's cell (its world, as JSON), runs the program a command at a
// time, posts each command's outcomes as it goes, and hands the cell
// back at the end.
import init, * as cell from '../cell-web.js';

const ready = init();

self.onmessage = async ({ data }) => {
  await ready;
  const { lines, world } = data;
  const wrong = cell.set_world(world);
  if (wrong) { self.postMessage({ error: 'the worker could not take the cell: ' + wrong }); return; }
  for (let k = 0; k < lines.length; k++) {
    self.postMessage({ k, n: lines.length, line: lines[k] });
    const r = JSON.parse(cell.command(lines[k]));
    self.postMessage({ k, r });
    if (r.error || r.outcomes.some((o) => !o.ok)) break;
  }
  self.postMessage({ done: true, world: cell.world() });
};
