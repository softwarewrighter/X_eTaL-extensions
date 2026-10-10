# How the voxel water flows

The water in the endless voxel world (`just demo scene voxels-water`)
is conserved: it moves from cell to cell and is never made or lost,
except at the sea. This page explains the rules simply, with the lines
of `extensions/scene/demos/Endless.xtl` that carry them out (the "Moving
water" section, from line 333). The longer, literate account -- the
rules built up from a row of five cups that run, the code drawn as
X_eTaL draws it, pictures from the run -- is
[Water that drains](https://softwarewrighter.github.io/X_eTaL-extensions/literate/water.html)
(its source: `docs/literate/water.org`).

## The idea

Think of every cell of moving water as a cup that holds up to 64
spoonfuls (`l:full := 64`, line 367): 64 is a full block, 32 half a
block. The world keeps a list of these cups -- each one's position, how
much it holds, and which way it was last moving -- as `en:w_ater`, an
n by 6 array of Ints (x y z, amount, dx dz). Water never appears or
disappears; spoonfuls only move from one cup to another. The one
exception is the sea, an endless bucket: it is made of ordinary blocks
from the terrain, not cups.

## One tick

Every other frame, one tick (`h:f_low`, line 377) does four things in
order.

1. The sea tops up its neighbors (`h:r_efill`, line 402). Any cup
   touching the sea at a side or from above is filled to 64 (line
   407). That is why a hole dug beside the sea fills.

2. Gravity first (`h:f_all`, line 413). Each cup pours down into what
   is below it: into air, or a cup with room, as much as fits (`room`,
   line 420); onto rock, nothing; into the sea, all of it, and it
   becomes sea (line 421).

3. Then spreading, in four passes -- west, east, north, south
   (`h:s_pread`, line 432). A cup spreads only when it cannot fall:
   rock or a full cup is under it (`held`, line 443). It gives water
   only to a neighbor holding less (`go`, line 447), a share of the
   difference set by a weight (`wt`, line 446):

   - 4/16 of the difference, normally;
   - 7/16 if it is already moving that way -- momentum, which is why a
     stream keeps going straight down a narrow trench instead of
     spreading round a terrace;
   - 8/16 if the neighbor has a drop under it -- water is pulled toward
     edges, so it pours off a step as a stream.

   The share is never more than half the difference (line 449), so two
   cups end up level instead of swapping. A difference of 2 or more
   always moves at least one spoonful; the front of a stream may push
   even one spoonful into an empty cell ahead of it (line 452), which
   is what lets a lake drain all the way instead of freezing on a
   gentle slope. A cup never takes more than it has room for, and never
   gives more than it holds (line 453).

4. Drawing (`h:d_rawWater`, line 468). Each cup is drawn see-through
   blue (scene's alpha), its top at its height (32 spoonfuls: half a
   block tall); only the surfaces that meet air are drawn.

## Asleep and awake

When a tick moves nothing, the water goes to sleep (`h:n_ext`, line
395) and costs nothing. Digging or placing a block (`l:s_et`, line 272),
pouring water (`l:p_our`, line 300), or a column being made again wakes
it. `l:s_tream` (line 116) shares the frames: each frame it does a third
of making a column of the world or one tick of water.

## Why a lake drains one layer at a time

The demo's lake is 25 cups a layer, two layers deep, on a stepped hill.
Dig the rim and the cups at the top layer's height can flow out through
the gap; every spoonful that leaves is gone from the lake, so its level
falls -- a layer for every 25 blocks of water that leave. When the top
layer is down to a thin film, nothing is above the breach's floor and
the flow stops. Dig the trench one block deeper and the layer below has
somewhere lower to go, so it drains the same way. The demo prints the
lake's layers each time the water comes to rest, and its golden test
pins those numbers: the top layer 25 blocks to 1.72, then, after the
deeper dig, the layer below 25 to 2.11. The water that left ran down
one face of the hill as a stream and into the sea.

## What it took

Two first tries failed in instructive ways:

- The four passes, giving the whole difference, made cups trade places:
  the west pass moved water one way, the east pass moved it back, and
  every tick ended where it began -- so the water fell asleep with the
  lake full. Capping a share at half the difference fixed it.
- With 8 spoonfuls a block, a lake rested on a slope of one spoonful a
  cell (a difference of 1 never moves, or water would slosh forever)
  and kept a third of its top layer. With 64 a block, and the front of
  a stream allowed to move by one, the slope left behind is a sliver.

## Not yet

Swimming works only below sea level (the player does not know about
moving water yet). Water that falls into a column not made yet stops
there, as on rock, until the column is made.
