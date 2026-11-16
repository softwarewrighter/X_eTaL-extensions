// The board's terrain: ground that is ground, a mesh the program worked
// out, and its material. A module of the board player (../board.js says
// what a module is). Three calls, all kept by their line:
//
//   terrain(name, nums('x z y ...', 3), ints('a b c ...', 3), nums('r g b ...', 3))
//     the terrain of that name: its corners (x east, z south, y up, in
//     tiles: the order tiles and pieces use), its triangles (three
//     corners each) and a color a corner.
//     - Corners count from 1 (FIRST, below): the first row of the
//       corners is corner 1, as X_eTaL counts, so a program writes the
//       indexes it worked out and subtracts nothing. A triangle naming a
//       corner there is not (0, or more than there are) is an error
//       that says which.
//     - Either winding: both sides of a triangle are drawn and lit, and
//       shadows fall from both. Keep one winding over a whole terrain
//       (the smooth normals are sums over a corner's triangles).
//     - A color is red, green and blue from 0 to 1, as the parts of a
//       '#rrggbb' color over 255 (0.37 0.58 0.31 is '#5f9550'). A corner
//       with no color is white.
//     Drawn with a standard material (so lights, shadows, fog and tone
//     mapping apply), normals smoothed over the triangles round a
//     corner, casting and taking shadows, by whatever camera the board
//     uses.
//
//   ground(name, 'grass', 'snow', nums('w ...', 1))
//     the material of the terrain of that name: two named textures and a
//     blend a corner, 0 the first and 1 the second (a corner with no
//     blend is 0). The corner colors of `terrain` still tint it: white
//     shows the texture as it is. Without a `ground` line a terrain is
//     its corner colors alone.
//     - A named texture is NAME.jpg, or NAME.png, in the textures folder
//       (options.textures; './textures/' beside the page by default;
//       null for no files) when there is such a file. Otherwise it is
//       one made here (terrain-textures.js: grass, snow, dirt, stone,
//       sand, mud), which is also what is drawn until a file has come.
//       A name with neither is plain white, and the console says so.
//     - A texture covers TILES by TILES tiles and repeats, laid on from
//       above, and from the side where the ground is steep, so it does
//       not stretch down a slope.
//     - Between two textures made here the edge is ragged, not a fade:
//       the second lies first in the hollows of the first (the fourth
//       number of a made texture's pixel is its height). 0 and 1 are
//       still wholly the first and wholly the second. Files fade evenly.
//
//   beyond(y, 'grass', '#8a9a70')
//   beyond(y, 'grass', '#8a9a70', nums('x0 z0 x1 z1', 4)[0])
//     ground that goes on to the horizon at that height, so a terrain
//     does not end in the air: a named texture as ground's (covering
//     TILES by TILES tiles, repeating) and a color that tints it. It is
//     drawn round the eye, wherever the eye goes (round the camera's
//     target for the board's own camera), a little behind anything at
//     its own height, and takes shadows and fog. A terrain whose edge
//     comes down to y meets it without a seam. The fourth argument is
//     a rectangle left out (west, north, east, south): where the
//     program's own terrain is, so a valley or a river bed in it lower
//     than y is not covered. A scene may hold several.
//
// terrain and ground may come in either order, and either may change alone.
// A changed `terrain` line replaces the mesh: when it has as many
// corners and triangles as before (ground pressed down, snow deformed)
// the numbers are written over the old ones, with no new buffers and no
// new shader; otherwise the buffers are made again, and still not the
// material. Either way most of the cost is reading the line's text
// (nums and ints: about 60 000 numbers for 10 000 triangles).
// A changed `ground` line writes the blends over the old ones.
import { SIZE, KNOWN, make } from './terrain-textures.js';

// the number of the first corner, in a triangle
const FIRST = 1;
// the tiles a texture covers before it repeats
const TILES = 4;

const DECLARE = `
#ifdef TERRAIN_GROUND
varying vec3 vTerrainAt;
varying vec3 vTerrainUp;
varying float vTerrainBlend;
#endif
`;
const VERTEX = `
#ifdef TERRAIN_GROUND
vTerrainAt = position;
vTerrainUp = normal;
vTerrainBlend = terrainBlend;
#endif
`;
// the two textures, each laid on along the three axes and weighed by the
// way the ground faces, then blended and multiplied into the color
const FRAGMENT = `
#ifdef TERRAIN_GROUND
{
  vec3 facing = pow(abs(normalize(vTerrainUp)), vec3(6.0));
  facing /= facing.x + facing.y + facing.z;
  vec3 at = vTerrainAt * terrainLook.x;
  vec4 first = texture2D(terrainA, at.xz) * facing.y + texture2D(terrainA, at.xy) * facing.z + texture2D(terrainA, at.zy) * facing.x;
  vec4 second = texture2D(terrainB, at.xz) * facing.y + texture2D(terrainB, at.xy) * facing.z + texture2D(terrainB, at.zy) * facing.x;
  float blend = clamp(vTerrainBlend, 0.0, 1.0);
  float high = mix(0.5, second.a, terrainLook.z) - mix(0.5, first.a, terrainLook.y);
  float ragged = smoothstep(-0.2, 0.2, blend - 0.5 + 0.3 * high);
  blend = mix(blend, ragged, max(terrainLook.y, terrainLook.z));
  diffuseColor.rgb *= mix(first.rgb, second.rgb, blend);
}
#endif
`;

export default function terrain(ctx) {
  const { THREE, scene, renderer, options } = ctx;
  const folder = options.textures === undefined ? './textures/' : options.textures;
  const fields = new Map(), looks = new Map();
  let running = true;

  const white = new THREE.DataTexture(new Uint8Array([255, 255, 255, 255]), 1, 1);
  white.colorSpace = THREE.SRGBColorSpace;
  white.needsUpdate = true;
  const tiling = (texture) => {
    texture.wrapS = texture.wrapT = THREE.RepeatWrapping;
    texture.colorSpace = THREE.SRGBColorSpace;
    texture.generateMipmaps = true;
    texture.minFilter = THREE.LinearMipmapLinearFilter;
    texture.magFilter = THREE.LinearFilter;
    texture.anisotropy = Math.min(4, renderer.capabilities.getMaxAnisotropy());
    texture.needsUpdate = true;
    return texture;
  };

  // A named texture: the one made here at once (white for a name not
  // known), and the file in its place when there is one.
  const look = (name) => {
    if (looks.has(name)) return looks.get(name);
    const pixels = make(name);
    const l = { texture: pixels ? tiling(new THREE.DataTexture(pixels, SIZE, SIZE)) : white, heights: pixels ? 1 : 0 };
    looks.set(name, l);
    const none = () => { if (!pixels) console.warn(`ground: no texture '${name}': no ${name}.jpg or ${name}.png in ${folder}, and the ones made here are ${KNOWN.join(', ')}`); };
    if (folder === null) { none(); return l; }
    const tries = ['jpg', 'png'];
    const next = () => {
      if (!tries.length) { none(); return; }
      const image = new Image();
      image.onload = () => {
        if (!running) return;
        if (l.texture !== white) l.texture.dispose();
        l.texture = tiling(new THREE.Texture(image));
        l.heights = 0;
        for (const f of fields.values()) dress(f);
      };
      image.onerror = next;
      image.src = `${folder}${name}.${tries.shift()}`;
    };
    next();
    return l;
  };

  const field = (name) => {
    if (!fields.has(name)) {
      fields.set(name, { name, shape: null, shapeLine: null, shapeNew: false, cover: null, coverLine: null, coverNew: false, drawn: null, material: null, uniforms: null, corners: 0, triangles: 0 });
    }
    return fields.get(name);
  };

  // a '#rrggbb' part over 255, as the renderer's own (linear) number
  const linear = (c) => (c <= 0.04045 ? Math.max(c, 0) / 12.92 : Math.pow((Math.min(c, 1) + 0.055) / 1.055, 2.4));

  // ground to the horizon: one square, kept round the eye in steps of a
  // texture's repeat, so the texture stays where it is on the world
  const WIDE = 1600;
  const far = new Set();
  const beyond = (y, name, color = '#ffffff', hole = null) => {
    const l = look(name);
    const g = new THREE.PlaneGeometry(WIDE, WIDE);
    g.rotateX(-Math.PI / 2);
    const at = g.attributes.position, uv = g.attributes.uv;
    for (let i = 0; i < at.count; i++) uv.setXY(i, at.getX(i) / TILES, at.getZ(i) / TILES);
    const material = new THREE.MeshStandardMaterial({
      color, map: l.texture, roughness: 0.95, polygonOffset: true, polygonOffsetFactor: 2, polygonOffsetUnits: 2,
    });
    if (hole) {
      // nothing is drawn inside the rectangle
      const rect = { value: new THREE.Vector4(hole[0], hole[1], hole[2], hole[3]) };
      material.onBeforeCompile = (shader) => {
        shader.uniforms.beyondHole = rect;
        shader.vertexShader = shader.vertexShader
          .replace('#include <common>', '#include <common>\nvarying vec2 vBeyondAt;')
          .replace('#include <begin_vertex>', '#include <begin_vertex>\nvBeyondAt = (modelMatrix * vec4(position, 1.0)).xz;');
        shader.fragmentShader = shader.fragmentShader
          .replace('#include <common>', '#include <common>\nuniform vec4 beyondHole;\nvarying vec2 vBeyondAt;')
          .replace('#include <map_fragment>', 'if (vBeyondAt.x > beyondHole.x && vBeyondAt.x < beyondHole.z && vBeyondAt.y > beyondHole.y && vBeyondAt.y < beyondHole.w) discard;\n#include <map_fragment>');
      };
    }
    const mesh = new THREE.Mesh(g, material);
    mesh.name = 'beyond';
    mesh.receiveShadow = true;
    mesh.frustumCulled = false;
    mesh.position.y = y;
    scene.add(mesh);
    const b = { mesh, material, l };
    far.add(b);
    return {
      dispose() {
        far.delete(b);
        scene.remove(mesh);
        g.dispose();
        material.dispose();
      },
    };
  };
  const follow = () => {
    for (const b of far) {
      // a texture's file may have come since
      if (b.material.map !== b.l.texture) { b.material.map = b.l.texture; b.material.needsUpdate = true; }
      const flat = ctx.camera === ctx.flat;
      const x = flat ? ctx.look.target[0] : ctx.camera.position.x, z = flat ? ctx.look.target[2] : ctx.camera.position.z;
      b.mesh.position.x = Math.round(x / TILES) * TILES;
      b.mesh.position.z = Math.round(z / TILES) * TILES;
    }
  };

  const kept = {
    beyond(line, y, name, color, hole) { return beyond(y, name, color, hole); },
    terrain(line, name, corners, triangles, colors = []) {
      const n = corners.length, m = triangles.length;
      const position = new Float32Array(n * 3), color = new Float32Array(n * 3).fill(1);
      for (let i = 0; i < n; i++) {
        const r = corners[i];
        position[i * 3] = r[0]; position[i * 3 + 1] = r[2]; position[i * 3 + 2] = r[1];
      }
      for (let i = 0, to = Math.min(n, colors.length); i < to; i++) {
        const r = colors[i];
        color[i * 3] = linear(r[0]); color[i * 3 + 1] = linear(r[1]); color[i * 3 + 2] = linear(r[2]);
      }
      const index = new (n > 65535 ? Uint32Array : Uint16Array)(m * 3);
      for (let t = 0; t < m; t++) {
        const r = triangles[t];
        for (let k = 0; k < 3; k++) {
          const c = r[k] - FIRST;
          if (!(Number.isInteger(c) && c >= 0 && c < n)) {
            throw new Error(`terrain('${name}'): triangle ${t + FIRST} names corner ${r[k]}; there are corners ${FIRST} to ${n - 1 + FIRST}`);
          }
          index[t * 3 + k] = c;
        }
      }
      const f = field(name);
      Object.assign(f, { shape: { position, color, index, n, m }, shapeLine: line, shapeNew: true });
      return { dispose() { if (f.shapeLine === line) Object.assign(f, { shape: null, shapeLine: null, shapeNew: true }); } };
    },
    ground(line, name, first, second, blends = []) {
      const f = field(name), blend = new Float32Array(blends.length);
      for (let i = 0; i < blends.length; i++) blend[i] = blends[i][0];
      Object.assign(f, { cover: { first, second, blend }, coverLine: line, coverNew: true });
      return { dispose() { if (f.coverLine === line) Object.assign(f, { cover: null, coverLine: null, coverNew: true }); } };
    },
  };

  // the normal at each corner: the sum of its triangles' (larger ones
  // count for more), made one long
  const normals = (position, index, out) => {
    out.fill(0);
    for (let t = 0; t < index.length; t += 3) {
      const a = index[t] * 3, b = index[t + 1] * 3, c = index[t + 2] * 3;
      const ux = position[b] - position[a], uy = position[b + 1] - position[a + 1], uz = position[b + 2] - position[a + 2];
      const vx = position[c] - position[a], vy = position[c + 1] - position[a + 1], vz = position[c + 2] - position[a + 2];
      const x = uy * vz - uz * vy, y = uz * vx - ux * vz, z = ux * vy - uy * vx;
      out[a] += x; out[a + 1] += y; out[a + 2] += z;
      out[b] += x; out[b + 1] += y; out[b + 2] += z;
      out[c] += x; out[c + 1] += y; out[c + 2] += z;
    }
    for (let i = 0; i < out.length; i += 3) {
      const long = Math.hypot(out[i], out[i + 1], out[i + 2]) || 1;
      out[i] /= long; out[i + 1] /= long; out[i + 2] /= long;
    }
  };

  const material = (f) => {
    f.uniforms = { terrainA: { value: white }, terrainB: { value: white }, terrainLook: { value: new THREE.Vector3(1 / TILES, 0, 0) } };
    f.material = new THREE.MeshStandardMaterial({ vertexColors: true, roughness: 0.95, metalness: 0, side: THREE.DoubleSide, shadowSide: THREE.DoubleSide });
    f.material.defines = {};
    f.material.onBeforeCompile = (shader) => {
      Object.assign(shader.uniforms, f.uniforms);
      shader.vertexShader = shader.vertexShader
        .replace('#include <common>', `#include <common>\n#ifdef TERRAIN_GROUND\nattribute float terrainBlend;\n#endif${DECLARE}`)
        .replace('#include <begin_vertex>', `#include <begin_vertex>${VERTEX}`);
      shader.fragmentShader = shader.fragmentShader
        .replace('#include <common>', `#include <common>\n#ifdef TERRAIN_GROUND\nuniform sampler2D terrainA;\nuniform sampler2D terrainB;\nuniform vec3 terrainLook;\n#endif${DECLARE}`)
        .replace('#include <map_fragment>', `#include <map_fragment>${FRAGMENT}`);
    };
  };

  // the terrain's material as its ground line says (or with none)
  const dress = (f) => {
    if (!f.drawn) return;
    const had = 'TERRAIN_GROUND' in f.material.defines;
    if (!f.cover) {
      if (had) { delete f.material.defines.TERRAIN_GROUND; f.material.needsUpdate = true; }
      f.uniforms.terrainA.value = f.uniforms.terrainB.value = white;
      return;
    }
    const a = look(f.cover.first), b = look(f.cover.second);
    f.uniforms.terrainA.value = a.texture;
    f.uniforms.terrainB.value = b.texture;
    f.uniforms.terrainLook.value.set(1 / TILES, a.heights, b.heights);
    if (!had) { f.material.defines.TERRAIN_GROUND = ''; f.material.needsUpdate = true; }
  };

  // after a scene's calls: each terrain as its two lines now say
  const settle = (f) => {
    if (f.shapeNew) {
      const s = f.shape;
      if (!s) {
        if (f.drawn) { scene.remove(f.drawn); f.drawn.geometry.dispose(); f.drawn = null; }
      } else {
        let g = f.drawn && f.drawn.geometry;
        if (g && f.corners === s.n && f.triangles === s.m) {
          // the same count of corners and triangles: written over
          g.attributes.position.array.set(s.position);
          g.attributes.color.array.set(s.color);
          g.index.array.set(s.index);
          g.attributes.position.needsUpdate = g.attributes.color.needsUpdate = g.index.needsUpdate = true;
        } else {
          if (g) g.dispose();
          g = new THREE.BufferGeometry();
          g.setAttribute('position', new THREE.BufferAttribute(s.position, 3));
          g.setAttribute('color', new THREE.BufferAttribute(s.color, 3));
          g.setAttribute('normal', new THREE.BufferAttribute(new Float32Array(s.n * 3), 3));
          g.setAttribute('terrainBlend', new THREE.BufferAttribute(new Float32Array(s.n), 1));
          g.setIndex(new THREE.BufferAttribute(s.index, 1));
          f.coverNew = true;
          if (!f.material) material(f);
          if (f.drawn) f.drawn.geometry = g;
          else {
            f.drawn = new THREE.Mesh(g, f.material);
            f.drawn.name = f.name;
            f.drawn.userData.terrain = true;
            f.drawn.castShadow = f.drawn.receiveShadow = true;
            f.drawn.frustumCulled = false;
            f.drawn.matrixAutoUpdate = false;
            scene.add(f.drawn);
          }
        }
        normals(g.attributes.position.array, g.index.array, g.attributes.normal.array);
        g.attributes.normal.needsUpdate = true;
        g.boundingBox = g.boundingSphere = null;
        f.corners = s.n; f.triangles = s.m;
        // the arrays are the geometry's now, or copied into it
        f.shape = { n: s.n, m: s.m };
      }
      f.shapeNew = false;
    }
    if (f.coverNew && f.drawn) {
      const blend = f.drawn.geometry.attributes.terrainBlend;
      blend.array.fill(0);
      if (f.cover) blend.array.set(f.cover.blend.subarray(0, Math.min(blend.array.length, f.cover.blend.length)));
      blend.needsUpdate = true;
      dress(f);
      f.coverNew = false;
    }
    if (!f.shape && !f.cover) {
      if (f.material) f.material.dispose();
      fields.delete(f.name);
    }
  };

  return {
    kept,
    end() { for (const f of [...fields.values()]) settle(f); },
    frame() { follow(); },
    count() {
      let terrains = 0, triangles = 0;
      for (const f of fields.values()) if (f.drawn) { terrains += 1; triangles += f.triangles; }
      return { terrains, terrainTriangles: triangles, beyond: far.size };
    },
    stop() {
      running = false;
      for (const f of fields.values()) {
        if (f.drawn) { scene.remove(f.drawn); f.drawn.geometry.dispose(); }
        if (f.material) f.material.dispose();
      }
      fields.clear();
      for (const b of [...far]) { scene.remove(b.mesh); b.mesh.geometry.dispose(); b.material.dispose(); }
      far.clear();
      for (const l of looks.values()) if (l.texture !== white) l.texture.dispose();
      looks.clear();
      white.dispose();
    },
  };
}
