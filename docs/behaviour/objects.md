# Placing objects from `.kq` files

Study session A, 2026-10-11. Answers `QUESTIONS.md` Q9 and Q10. Sources:
`Mask.exe` (studied, described in our own words) and the data files.

## Orientation (Q9)

1. An object's placement is read as: first a full matrix if a key
   `<prefix>RMat` exists (flags followed by nine numbers); otherwise the
   three angles `dirX`, `dirY`, `dirZ`; then the position `locX`, `locY`,
   `locZ`. No `.kq` file in `daventry/` or `GAME/` uses `RMat`, so in
   practice it is always the three angles. (Traced.)
2. Angles are **radians** (they feed the same angle-to-rotation function
   used everywhere; data values like 1.57 and 3.14 agree).
3. Positive angles follow the right-hand rule: looking from the positive end
   of the axis back towards the origin, a positive angle turns
   anticlockwise. So positive `dirZ` turns +X towards +Y (anticlockwise seen
   from above), positive `dirX` turns +Y towards +Z, positive `dirY` turns
   +Z towards +X. (Traced.)
4. Order, with row vectors (`point × matrix`, see `models-and-textures.md`):
   the rotation matrix is `RotY(dirY) × RotX(dirX) × RotZ(dirZ)`, i.e. a
   point is turned about **Y first, then X, then Z**. (Traced.)
   With column vectors this is `RotZ · RotX · RotY`.
5. **No scale or size field**: the placement transform has no scale, and no
   object file uses a scale key.
6. `rotMinX/Y/Z` / `rotMaxX/Y/Z` belong to particle emitters (`KQEmitter`,
   which also has a `KQEmitter::setRotMin` script command). They are not
   object orientation; ignore them for placement.

## Height (Q10)

1. **`isGroundRelative` is never read by the game**: the name does not occur
   anywhere in `Mask.exe` (case-insensitive search). Treat it as a leftover
   from the level editor. `locZ` is used as written.
2. Measured in Daventry (`KQInterior` sections with that flag): either
   `locZ` is −1/−2, or it is an absolute height 10–35 units above the ground
   under the object (barn, docks, pub; `look` is 1,607 above).
   **Answered in `buildings.md`**: `keepOnGround` is on by default and
   raises anything below the ground up onto it, so use
   `max(locZ, ground height at locX, locY)`.
3. **`keepOnGround`** is read together with `useGravity`, `submerged` and
   `MaxSlopeDot`. Default on. Rule (see `buildings.md`): if the object is
   below the ground it is raised to the ground, never lowered. An object
   whose `.kq` says `keepOnGround=0` keeps its `locZ`.
4. Related, for M2: `MaxSlopeDot` defaults to 0.819 = cos 35°: ground whose
   normal is more than about 35° from straight up is too steep to walk on.
   (Default traced; how it is used not traced yet.)
