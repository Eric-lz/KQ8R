# Saved games and binary `PERS` format

Study session A, 2026-11-24. Sources: the program (traced in `Mask.exe`), the game's data and scripts in `MASK/`, and all 24 real saved games in `MASK/save/save1.vol`–`save24.vol` (read-only; 9,686 binary `PERS` chunks verified byte-for-byte with 0 failures). Answers `Q207` in `docs/behaviour/QUESTIONS.md`. Builds on [submenus.md](submenus.md) (rules 3–11) and [world-change.md](world-change.md) (rules 6–20).

## Purpose

Describe the complete on-disk format of King's Quest: Mask of Eternity saved games (`save/save<N>.vol`) and kept-world archives (`cur/<World>.vol`), including the `PVOL` container, the dual binary (`PERS`) / ASCII (`.kq` INI) object persistence format, the exact binary layout of every saved class, and how `newWorld.cs` restores mid-world state without replaying first-time cutscenes.

## Rules

### 1. Archive container (`PVOL`) and entry layout

1. **Both `save<N>.vol` and `cur/<World>.vol` use the uncompressed `PVOL` archive format:**
   - **Header (8 bytes):** ASCII `"PVOL"` followed by a 32-bit little-endian byte offset to the `"vols"` string table.
   - **File blocks:** One `"VBLK"` block per entry:
     - 4-byte tag `"VBLK"`,
     - 4-byte little-endian `size | 0x80000000` (bit 31 set means 4-byte alignment padding),
     - `size` raw payload bytes,
     - `0` to `3` zero padding bytes so the next block begins on a 4-byte boundary.
   - **Name table (`"vols"`):** 4-byte tag `"vols"`, 4-byte little-endian byte length, concatenated null-terminated ASCII entry names, padded with `0` to `3` zero bytes to a 4-byte boundary (note: original saves pad to an even 2-byte boundary, so readers should scan up to 3 bytes after `"vols"` for `"voli"`).
   - **Index (`"voli"`):** 4-byte tag `"voli"`, 4-byte little-endian byte length (`count × 17`), followed by 17-byte records:
     - `u32` unused (`0`),
     - `u32` byte offset of the entry's name in `"vols"`,
     - `u32` byte offset of the entry's `"VBLK"` header from file start,
     - `u32` payload byte size,
     - `u8` compression method (`0` = uncompressed; all saved games use `0`).
2. **Entries inside `save/save<N>.vol` (in write order):**
   1. `savedGameName` — raw ASCII bytes of the save's display name (no null terminator).
   2. `worldName` — raw ASCII bytes `<World>.kq` (e.g., `Daventry.kq`, `Swamp.kq`).
   3. `savedGameVersion` — raw ASCII bytes `"1.000000"`.
   4. **Every persistent object in the current world** as `<objectName>.kq` in **binary `PERS`** format, in the exact order held by the persistence manager: `Terrain.kq` first, `KQCamera.kq` second, then `WorldInventory.kq`, `Music.kq`, and every live persistent object (characters, doors, chests, traps, animated objects, portals, point lights, terrain animators, etc.).
   5. `worldState.kq` — **binary `PERS`** `KQWorldState` object containing all pending timed `KQEvent`s in the event queue (e.g., scheduled `KQCueEvent`s).
   6. `WorldVar.cs` — plain-text console script of `set <var> <value>` lines for all variables starting with `<World>` or `KQWorld` (except `KQWorld::SaveGameName`).
   7. `newWorld.cs` — generated plain-text console script that restores the world clock, variables, terrain, Connor, camera, map, permanent world objects, every saved object, and `WorldState.kq`.
   8. `current.bin` — 19,204-byte screenshot thumbnail (`u32` palette ID + `160 × 120` 8-bit indices; see [submenus.md](submenus.md) rule 7).
   9. `Connor.kq` — **binary `PERS`** `KQConner` object (full player state, position, stats, inventory, spells, flags, animation lists, skin lists).
   10. `<map>.kq` — **binary `PERS`** `KQMap` object for the current world (e.g., `davMap.kq`, `swmpMap.kq`).
   11. `GameState.kq` — **binary `PERS`** `KQGameState` object (`nextNoName` counter and `difficultyFactor`).
   12. Every kept-world archive `<OtherWorld>.vol` from `cur/*.vol` (each itself a complete `PVOL` archive), followed by `worldVolumes` (pipe-separated ASCII list of those `.vol` filenames, or 0 bytes if none).
   13. Every kept map `<otherMap>.kq` from `cur/*.kq` (excluding the current world's map), followed by `maps` (pipe-separated ASCII list of those `.kq` filenames, or 0 bytes if none).

### 2. Differences between `save<N>.vol` and `cur/<World>.vol` (kept worlds)

3. **Binary `PERS` vs ASCII `.kq` INI.**
   - When the player saves from the **Save Game** menu (`save<N>.vol`), the current world's objects, `worldState.kq`, `Connor.kq`, `<map>.kq`, and `GameState.kq` are serialized in **binary `PERS`** format.
   - When Connor crosses a **portal** into another world, the departed world's snapshot (`cur/<World>.vol`), its map (`cur/<map>.kq`), and the hand-over `cur/Connor.kq` and `cur/GameState.kq` are serialized in **ASCII `.kq` INI** format (`[ObjectName]\r\nclassType=...\r\nkey=value\r\n...`).
   - Consequently, a multi-world `save<N>.vol` contains **binary `PERS`** chunks for the current world, while its embedded `<OtherWorld>.vol` archives and `<otherMap>.kq` files (copied verbatim from `cur/`) contain **ASCII `.kq` INI** files.
   - **Important for two-way compatibility:** Every class uses a single unified serializer for both binary `PERS` and ASCII `.kq` INI. On load, the loader inspects the first 4 bytes of **every** `.kq` file: if they are `"PERS"` (or `"DICT"`), it reads binary `PERS`; otherwise it rewinds to byte 0 and parses ASCII `.kq` INI by matching `classType=<ClassName>`. Therefore, the original `Mask.exe` seamlessly loads **either** binary `PERS` or ASCII `.kq` INI files inside `save<N>.vol` and `cur/<World>.vol`.

### 3. Binary `PERS` container and primitive types

4. **`PERS` block framing.** Every binary persistent object is wrapped in an IFF-style block:
   - `4 bytes`: ASCII `"PERS"`.
   - `4 bytes`: `u32` little-endian `payload_size` (bit 31 is `0`, meaning the block is **2-byte aligned** on disk).
   - `2 bytes`: `u16` little-endian `class_name_len` (`1..=128`, without null terminator).
   - `((class_name_len + 1) & ~1)` bytes: ASCII class name (e.g., `KQConner`, `KQGameState`), padded with a single `0x00` byte when `class_name_len` is odd.
   - **No version field** is written (every KQ8 class uses version `0` with empty version read/write).
   - **Fields:** Serialized sequentially in little-endian binary with no key names or section headers.
   - **Block end padding:** If `payload_size` is odd, 1 zero byte follows the block so the next sibling block begins on a 2-byte boundary.
5. **`DICT` back-reference block.** During a single top-level store/load, an object dictionary records every pointer written/read in order (`0`-based). If a previously stored object pointer is encountered again, the writer emits a `"DICT"` block (`"DICT"`, `u32(4)`, `u32(index)`) instead of duplicating the object. (In practice, across all 24 inspected saves, `DICT` never appears in saved games because object trees are non-shared.)
6. **Binary primitive encodings:**
   - **`i32` / `u32`:** 4-byte little-endian integer. Smaller integer fields (`u8` collision shape/detail/priority, `u16` group masks, `immunities`, `EnvType`) are widened to 4-byte `u32` in the binary stream.
   - **`f32`:** 4-byte little-endian IEEE-754 single-precision float.
   - **`Point3F` / `EulerF`:** 3 consecutive `f32` values (`x, y, z` or `rx, ry, rz`, 12 bytes).
   - **`TMat3F` (transform matrix, 52 bytes):** `u32 flags`, followed by 9 `f32` values (3×3 row-major rotation matrix), followed by `Point3F` (`locX, locY, locZ`).
   - **`string`:**
     - If null/absent: `u32(0)` (4 bytes total).
     - If present: `u32(padded_len)` where `padded_len = ((strlen + 1) + 1) & ~1` (i.e., `strlen + 1` rounded up to an even number), followed by the ASCII characters, a `0x00` terminator, and an extra `0x00` pad byte if `strlen + 1` was odd.
   - **`string[]` (string array):** `i32 count`, followed by `count` `string`s.
   - **`Point3F[]` (point array):** `i32 count`, followed by `count` `Point3F`s.
   - **`KQEvent`:** Serialized as a single `string` (up to 500 bytes), e.g., `"KQCueEvent Connor 308.749695 2300 144 (null)"`.
   - **Named nested object (`NestedNamed`):** First a `string` holding the sub-object's name (or `u32(0)` if the pointer is null). If non-null, immediately followed by a nested `"PERS"` (or `"DICT"`) block.

### 4. Exact binary `PERS` layouts by class

Every subclass begins with all fields of its parent class in exact inheritance order.

#### Base classes

7. **`KQBase`:**
   - `name` (`string`) — the object's unique name in the world (e.g., `"Connor"`, `"Condoor"`, `"GameState"`).
8. **`KQVisibleBase` (inherits `KQBase`):**
   - `active3DFlag` (`i32`), `isFar` (`i32`), `tooFar` (`f32`), `translucency` (`i32`), `hazeMultiplier` (`f32`), `b_hazeSolid` (`i32`), `hidden` (`i32`).
9. **`KQMover` (inline or standalone `PERS` inheriting `KQVisibleBase`):**
   - Inline mover fields (68 bytes): `transform` (`TMat3F`, 52 bytes), `keepOnGround` (`i32`), `useGravity` (`i32`), `submerged` (`i32`), `MaxSlopeDot` (`f32`).
   - Standalone `KQMover` `PERS` block: `KQVisibleBase` + inline mover fields.
   - **`KQAnimMover` and `KQAnimFlyingMover`** (inherit `KQMover`): `rotateSpeed` (`f32`), `fast` (`i32`), `lookAt` (`Point3F`), `useLookAt` (`i32`).

#### Global and world-level files

10. **`GameState.kq` (`KQGameState`, inherits `KQBase`):**
    - `nextNoName` (`i32`) — next integer for auto-generated `NoName<N>` object names.
    - `difficultyFactor` (`f32`) — combat difficulty multiplier (`0.66` Easy, `1.0` Normal, `1.5` Hard).
11. **`worldState.kq` (`KQWorldState`, inherits `KQBase`):**
    - `nEvents` (`i32`), followed by `nEvents` `KQEvent` strings (all pending timed events in the world event queue, re-queued on load).
    - **Correction to [submenus.md](submenus.md) rule 10.2:** `worldState.kq` does **not** hold world flags or camera mode; it holds only the pending timed event queue.
12. **`Music.kq` (`KQMusic`, inherits `KQVisibleBase`):**
    - `nMusic` (`i32`), followed by `nMusic` `string`s (active music stack filenames, e.g., `"forest.aud"`).
13. **`KQCamera.kq` (`KQCamMover`, inherits `KQVisibleBase`):**
    - `followObjectName` (`string`), `pauseFollow` (`i32`),
    - 6 `f32`: `nearPlane`, `farPlane`, `fov`, `tiltflag`, `zoomflag`, `panflag`,
    - `fixHeight` (`i32`),
    - 24 `f32`: `oldpanflag`, `oldzoomflag`, `oldangDiffZ`, `camMoveRadius`, `shortcamMoveRadius`, `longcamMoveRadius`, `camRotateAmount`, `connerMinDist`, `camFlyDist`, `angTotal`, `angStep`, `rollStep`, `camConLen`, `shakeDuration`, `xShakeMax`, `zShakeMax`, `xShakeAmp`, `zShakeAmp`, `oldCMRX`, `conHeadHeight`, `camHeight1`, `camHeight2`, `xShakePeriod`, `zShakePeriod`,
    - 5 `Point3F`: `loc`, `oldcamaxis`, `cameraOffset`, `thirdcameraOffset`, `moveStep`,
    - 4 `EulerF`: `cameraRotation`, `cameraRotationDefault`, `dir`, `thirdcameraRotation`,
    - 2 `TMat3F`: `connorMat`, `oldconnorMat`,
    - 22 `i32`: `camRotModeX`, `camRotModeZ`, `conRelMove`, `oldconRelMove`, `checkmove`, `collisionRadius`, `oldcamRotModeZ`, `moveCam`, `aimCam`, `inPan`, `firstpan`, `firstzoom`, `touchedTerrain`, `isFirstPerson`, `constop`, `nMoveStep`, `zipCam`, `angRot`, `actionCam`, `shake`, `shakeFrame`, `userHeight`.
14. **`<map>.kq` (`KQMap`, inherits `KQVisibleBase`):**
    - 8 `i32`: `north`, `west`, `south`, `east`, `useFog`, `fogRadius`, `holdMapLoc`, `isActive`,
    - 4 `string`: `blankMap`, `fullMap`, `arrowPBA`, `mapName`,
    - `earnedTeleport` (`i32`), `TeleportScript` (`string`), `TeleportPortalKQ` (`string`),
    - `cMapIcons` (`i32`), followed by `cMapIcons` records of `(mapIcon: string, srcIcon: string, xIcon: i32, yIcon: i32)`,
    - Fog-of-war RLE stream: sequence of `i32` run lengths alternating cleared/fogged bits (MSB-first across `460` rows × `58` bytes = `460 × 464` bits), terminated by `i32(-1)`.
15. **`Terrain.kq` (`KQTerrain`, inherits `KQBase`):**
    - 6 `string`: `materialListFile`, `materialBMP`, `heightBMP`, `heightBMP2`, `heightBMP3`, `heightBMP4`,
    - `heightBMPScale` (`i32`), `materialDirection` (`string`), `floorBMPName` (`string`),
    - 3 `f32`: `visibleDistance`, `perspectiveDistance`, `hazeDistance`,
    - `groundScale` (`i32`), `smoothing` (`i32`),
    - 12 `f32`: `detailTable0min/max` .. `detailTable5min/max`,
    - `blockSize` (`i32`), `lightScale` (`i32`),
    - 6 `string`: `envinfoName`, `tedfileName`, `lightmapName`, `shadowmapName`, `SWlightmapName`, `SWshadowmapName`,
    - `nDeactivatedLights` (`i32`), followed by `nDeactivatedLights` records of `(fType: i32, fRed: f32, fGreen: f32, fBlue: f32, fWPosition: Point3F, fRange: f32, fFalloff: f32)`,
    - `nChangedHeights` (`i32`), followed by `nChangedHeights` records of `(h_startx: i32, h_starty: i32, h_endx: i32, h_endy: i32)`.
16. **`WorldInventory.kq` (`KQWorldInventory`, inherits `KQInventory` -> `KQBase`):**
    - `ownerId` (`u32`, `0xFFFFFFFF` for world inventory),
    - `numItems` (`i32`), followed by `numItems` `NestedNamed` `KQInventoryItem` blocks (every loose/dropped item lying in the world, such as dropped coins and picked-up/dropped gear).

#### World objects (`KQObject` hierarchy)

17. **`KQObject` (inherits `KQVisibleBase`):**
    - `shapeName` (`string`), `acceptsInventory` (`string`),
    - 5 `u32`: `colliderType`, `colliderMask`, `collisionShape`, `collisionDetail`, `collisionPriority`,
    - 2 `f32`: `stepUpAmount`, `stepDownAmount`,
    - 5 `u32`: `groupsAlways`, `groupsActive3D`, `groupsActive3DFar`, `groupsInvisible`, `immunities`,
    - `acceptAllItems` (`i32`), `alternateBmp` (`string`), `originalBmp` (`string`), `dithered` (`i32`), `ouchSound` (`string`), `script` (`string`),
    - 4 `i32`: `shadowvisible`, `conCollisionAction`, `worldCollisionAction`, `tooFarAwayAction`,
    - `shadowmask` (`u32`), 3 `i32`: `doNotPointAt`, `lockedResources`, `footsoundType`, `ouchTime` (`f32`),
    - `projectileFile` (`string`), `shootOffset` (`Point3F`), `relAimPoint` (`Point3F`), `objProjDamage` (`i32`), `lastWeaponType` (`i32`),
    - `EnvType` (`u32`),
    - `stats` (`string`: 5 space-separated integers `"health maxHealth experience damage protection"`),
    - `moverType` (`i32`):
      - If `0`: `moverName` (`string`), followed by inline `KQMover` fields (68 bytes).
      - If non-zero (`1`): `NestedNamed` mover `PERS` block (e.g., `KQAnimMover`, `KQAnimFlyingMover`).
18. **`KQInventoryItem` (inherits `KQObject`):**
    - `quantity` (`i32`), `userId` (`u32`), `timeLeft` (`f32`),
    - `itemType`: in **binary `PERS`** an `i32` (the numeric `ID #` from `Invinfo.itl`); in **ASCII `.kq`** a string (the `INVITEM_*` name).
19. **`KQInterior` (inherits `KQObject`):**
    - 4 `string`: `material`, `swLightmapName`, `altswLightmapName`, `althwLightmapName`,
    - `usingAltLighting` (`i32`), `trapCollider` (`NestedNamed` `KQTrap` or null).
20. **`KQAudibleObject` (inherits `KQObject`):**
    - `soundFileName` (`string`), `isSound3D` (`i32`), `minDistance` (`f32`), `maxDistance` (`f32`), `delta` (`Point3F`).
21. **`KQSwampTile` (inherits `KQObject`):** `heightNum` (`i32`), `damFactor` (`f32`).
22. **`RiverControl` (inherits `KQObject`):** `BBMin` (`Point3F`), `BBMax` (`Point3F`), `stretchDist` (`f32`), `dir` (`i32`), `rustedOne` (`i32`).
23. **`KQAnimObject` (inherits `KQObject`):**
    - `animPos` (`f32`, current phase `0.0..1.0` in the active movement),
    - 4 `string`: `currAnimName`, `nextAnimName`, `preTransAnimName`, `postTransAnimName`,
    - 2 `f32`: `nextAnimStartTime`, `postTransAnimStartTime`,
    - `flags` (`u32`), `flyingObject` (`i32`), `turnOffDynLighting` (`i32`),
    - `animLoops` (`NestedNamed` `KQAnimLoopList`),
    - `ownsAnimList` (`i32`), `notifyObject` (`string`).
    - **`KQAnimLoopList` (inherits `KQBase`):** `nMovement` (`i32`), followed by `nMovement` records of `(movementName: string, loopSpec: string, nCue: i32, cues: string[nCue], nTransition: i32, transitions: string[nTransition])`.
24. **`KQAnimOnce` (inherits `KQAnimObject`):** `deleteWhenDone` (`i32`).
25. **`KQDoor` (inherits `KQAnimObject`):** `state` (`i32` — door state; open/closed mesh and animation phase come from `KQAnimObject`'s `currAnimName` and `animPos`, while `state` tracks the door state machine). Note: wall switches such as Daventry's `arroswt1`–`3` are `KQDoor` instances; there is no `KQSwitch` class (`KQSwitchPlate` is the pressure plate class).
26. **`KQChest` (inherits `KQAnimObject`):**
    - `invItemTypeName` (`string`), `invQuantity` (`i32`), `invLoc` (`Point3F`), `invDir` (`EulerF`), `chestState` (`i32`), `invItemName` (`string`),
    - 6 `i32`: `TakeMsg`, `TakeNoun`, `TakeVerb`, `TakeCase`, `TakeSeq`, `TakeEnd`,
    - 6 `i32`: `EmptyMsg`, `EmptyNoun`, `EmptyVerb`, `EmptyCase`, `EmptySeq`, `EmptyEnd`.
27. **`KQPortal` (inherits `KQAnimObject`):**
    - 6 `string`: `world`, `enterAnim`, `exitAnim`, `enterAnmFile`, `exitAnmFile`, `exitScript`,
    - `destConnor` (`TMat3F`), `destCam` (`TMat3F`), `noLoc` (`Point3F`).
28. **`KQBoulder` (inherits `KQAnimObject`):** `prevPos` (`Point3F`).
29. **`KQLavaTile` (inherits `KQAnimObject`):** `holdflag` (`i32`), `topbmp` (`string`), `reversed` (`i32`), `metal` (`i32`), `unknown` (`i32`), `activated` (`i32`).

#### Traps, lights, hazers, terrain animators

30. **`KQTrap` (inherits `KQVisibleBase`):**
    - `loc` (`Point3F`), `trapType` (`u32`), `innerPercent` (`f32`), `script` (`string`), `colliderMask` (`u32`), `BBMin` (`Point3F`), `BBMax` (`Point3F`), `lookupCall` (`i32`), `shrunk` (`i32`), `trappedObject` (`string[]` — names of objects currently inside the trap).
31. **`KQSpiriTrap` (inherits `KQTrap`):** `spiritState` (`i32`), `trapStatus` (`i32`), `damage` (`f32`).
32. **`KQHazer` (inherits `KQVisibleBase`):** `loc` (`Point3F`), `innerRadius` (`f32`), `colliderMask` (`u32`), `radius` (`f32`), `collided` (`i32`), `hazeCenterFactor` (`f32`), `origHazeEdge` (`f32`), `hazeEdge` (`f32`).
33. **`KQPointLight` (inherits `KQVisibleBase`):** `tsLightType` (`i32`), 3 `f32` position (`x, y, z`), 3 `f32` intensity (`red, green, blue`), 2 `f32` attenuation (`begin, end`).
34. **`KQTerrainAnimator` (inherits `KQVisibleBase`):**
    - `startNow` (`i32`), `repeatflag` (`i32`), `frameCount` (`i32`), `delay` (`f32`),
    - `numMaterialIndex` (`i32`) + `i32[numMaterialIndex]`,
    - `numMaterialIndexII` (`i32`) + `i32[numMaterialIndexII]`,
    - `numGridBlocks` (`i32`) + `(x: i32, y: i32)[numGridBlocks]`,
    - **Only if `numGridBlocks == 0`**: `numOrigTextures` (`i32`) + `i32[numOrigTextures]`, `numExclude` (`i32`) + `(x: i32, y: i32)[numExclude]`.

#### Characters (`KQMonster` and `KQConner`)

35. **`KQMonster` (inherits `KQAnimObject`):**
    - `state` (`NestedNamed` `KQState*`), `nextState` (`i32`), `nextStateData` (`string`),
    - `stateChangeNotifyGroupName` (`string`), `stateChangeMembers` (`string[]`), `dependents` (`string[]`), `guardingDependent` (`i32`),
    - `monTickCount` (`i32`), `ticksSinceSeenConnor` (`i32`), `monsterType` (`string`), `alarmRadius` (`f32`),
    - 3 `i32`: `returningToHomeRadius`, `bMustGoHome`, `keepingDistance`,
    - 6 `f32`: `homeRadius`, `chaseRadius`, `attackDist`, `attackTooClose`, `attackFarEnough`, `likelyToFlee`,
    - `mustFlee` (`i32`), 6 `f32`: `attackDelay`, `birthDelay`, `alarmTime`, `attackTime`, `chaseTime`, `reactTime`,
    - `hitsSinceAlarmed` (`i32`), `doMode` (`i32`), `doLoc` (`Point3F`), `lookedUpProjectile` (`i32`),
    - `connerDist` (`f32`), `connerTime` (`f32`), `lastConnerPos` (`Point3F`), `canSeeConner` (`i32`), `canAlwaysSeeConner` (`i32`), `lineOfSightOffset` (`f32`), `conVec` (`Point3F`),
    - `inGoodBye` (`i32`), `noTurn` (`i32`), `startLoc` (`Point3F`), `minWanderDist` (`f32`),
    - 4 `string`: `deadDts`, `idedKqs`, `projdKqs`, `deadBmp`,
    - `normalReactPercent` (`f32`), `specialReactAnim` (`string`), `specialReactPercent` (`f32`), `specialConnorReactAnim` (`string`),
    - `idleTime` (`f32`), `idling` (`i32`), `active` (`i32`),
    - 7 `string`: `moveAnim`, `lastAnimPlayed`, `animFile`, `idleAnimFile`, `savedAnimList`, `invDrop`, `droppedWeapon`,
    - `selfMember` (`i32`), `monUseGravity` (`i32`),
    - 22 `i32`: `GB` (5), `WB` (5), `Alarm` (6), `Surprise` (6) message tuples,
    - `SaidAlarmMessage` (`i32`), `chaseObject` (`string`), `team` (`string`),
    - `wanderPts` (`Point3F[]`), `wanders` (`i32`), `wanderAndStop` (`i32`), `curWanderPt` (`i32`), `shootOffsets` (`string[]`),
    - `inventory` (`NestedNamed` `KQInventory` or `KQConnerInventory`),
    - `baseLip` (`string`), `lips` (`NestedNamed` `KQLips`: `KQBase` + 8 `string`s `lip0`..`lip7`),
    - `Conversation` (`NestedNamed` `KQConversation`: `KQVisibleBase` + 3 `f32` `convLocX/Y/Z` + 2 `string[]`),
    - `Talker` (`NestedNamed` `KQTalk`: `KQVisibleBase` + 3 `i32` `flags/nostop/noFade` + `talkerName` `string` + `talkListSize` `i32` + `talkList` strings `"<file> <noun> <verb> <case> <seq> <id> <flags>"`).
36. **Monster subclasses:**
    - `KQGeometryTalker`, `KQDeath`: no extra fields over `KQMonster`.
    - `KQTowerSkeleton`: 5 `f32` (`realAlarm`, `moveTime`, `timeToRun`, `stunPause`, `reactPause`), `bGreetedConnor` (`i32`), `oldCamPos` (`Point3F`), `bReactingBack` (`i32`), `bScreamed` (`i32`).
    - `KQMandragorRoot`: `offset` (`Point3F`), `origHealth` (`i32`), 4 `f32` (`origBirthDelay`, `rebirthDelay`, `repelDist`, `tooClose`).
    - `KQMandragorTree`: `rootDownCount` (`i32`), `maxRoots` (`i32`).
37. **`Connor.kq` (`KQConner`, inherits `KQMonster`):**
    - 4 `f32`: `zoomULY`, `zoomULX`, `zoomLRY`, `zoomLRX`,
    - `allowShadowsInGame` (`u32`),
    - `nextDoMode` (`i32`), `requiredWeaponMode` (`i32`), `requiredWeaponScript` (`string`),
    - 14 `i32`: `handsOff`, `guiBottom`, `talkguiBottom`, `handsOffCursor`, `animSetHandsOff`, `solvedTile3`, `connorExited`, `firstperson`, `tilePuzzleJump`, `inWeaponDoMode`, `AttackFireFrame`, `bowAttacking`, `bWalkToLoc`, `bCanChangeFPMode`,
    - `walkToLocation` (`Point3F`),
    - 7 `i32`: `PointCueFlag`, `PointCueI1`, `PointCueI2`, `WalkCueFlag`, `RunCueFlag`, `WalkCueI1`, `WalkCueI2`,
    - 2 `f32`: `maxDamageFactor`, `minDamageFactor`,
    - 3 `i32`: `runtowalk`, `terrainSound`, `ropeStatus`,
    - 5 `Point3F`/`EulerF`: `ropeLoc`, `ropeDir`, `ropeCamLoc1`, `ropeCamLoc2`, `ropeCam2Loc`,
    - 7 `f32`: `ropeCamAng`, `ropeCamRange`, `ropeCamTopLimit`, `ropeCamBottomLimit`, `boxUpTime`, `waitExitTime`, `lastplayedscroll`,
    - `lockedStateId` (`i32`), `lockedStateData` (`string`),
    - 5 `i32`: `wasInFirst`, `wasInThird`, `startHelp`, `worldFlags`, `wasInBowMode`,
    - 2 `string`: `PointCueObj`, `WalkCueObj`,
    - `allowLeaning` (`i32`), `level` (`i32`), `itemInHand` (`string`),
    - 9 `i32`: `claritySpell`, `strengthSpell`, `invulnerabilitySpell`, `invisibilitySpell`, `strengthSpellPoints`, `armorBeforeSpell`, `altShort`, `altLong`, `altArmor`,
    - `talkMonName` (`string`), `talkEnemy` (`i32`), `talkDone` (`i32`), `talkMonLoc` (`Point3F`),
    - 6 `NestedNamed` `KQConAnimList` (`KQBase` + `nAnimList: i32` + `string[nAnimList]` `"<itemType> <anmFile>"`): `AnimList`, `ShortAnimList`, `LongAnimList`, `FpAnimList`, `ShortFpAnimList`, `LongFpAnimList`,
    - 3 `NestedNamed` `KQSkinList` (`KQBase` + `nSkin: i32` + `string[nSkin]` `"<bmp> <mask>"` + `nInvItems: i32` + `string[nInvItems]`): `skinList`, `skinList_firstperson`, `lipsList`,
    - 4 `i32`: `curSkin`, `oldSkin`, `curLips`, `oldLips`,
    - `holdHelpSize` (`i32`), followed by `holdHelpSize` decimal-integer `string`s.
38. **Common `KQState` subclasses** (each starts with `KQBase` + `fromState: i32`):
    - `KQStateWait`, `KQStateConnerWait`: `conRadius` (`f32`), `monRadius` (`f32`).
    - `KQStateMove`: `conRadius` (`f32`), `monRadius` (`f32`), `startedMoving` (`i32`).
    - `KQStateUnborn`: `hasPreBirth` (`i32`), `readyToBirth` (`i32`).
    - `KQStateConnerPortal`: `enteredPortal` (`i32`), `donePortal` (`i32`), `processedGUI` (`i32`), plus 7 `string`s (`portalName`, `trapName`, `portalEnterAnim`, `portalExitAnim`, `portalEnterAnmFile`, `portalExitAnmFile`, `portalExitScript`).
    - `KQStateConnerHookRope`: 3 `i32` (`doneRoping`, `reachedTop`, `doModeBefore`), 2 `Point3F` (`oldCamPos`, `conPos`), 3 `i32` (`onRope`, `hideIcon`, `firstToThird`), 4 `f32` (`bottomCamLimit`, `topCamLimit`, `conCamBottomLimit`, `conCamTopLimit`), 2 `Point3F` (`ropeCamLoc1`, `ropeCamLoc2`), 2 `f32` (`origStepUp`, `origStepDown`).
    - `KQStateConnerSpecial`: `animCommandString` (`string`), `curAnimList` (`string`), `animList` (`string`).
    - `KQStateConnerPush`: `pushed` (`i32`). `KQStateConnerFall`: `doneFalling` (`i32`). `KQStateConnerJump`: `doneJumping` (`i32`).

### 5. How `newWorld.cs` restores mid-world state without replaying opening cutscenes

39. **Generated `newWorld.cs` sequence.** When saving a world (either to `save<N>.vol` or `cur/<World>.vol`), the persistence manager writes `newWorld.cs`:
    1. `setCurrentTime <float>` — restores the world clock to the exact second it was saved.
    2. `WorldVar.cs` — restores all saved `<World>::*` and `KQWorld::*` variables.
    3. `loadKQ Terrain.kq`, `loadKQ Connor.kq`, `loadKQ KQCamera.kq`, `loadKQ <map>.kq`.
    4. `permWrld.cs $KQGame::runOptimal` — loads all permanent, non-persistent scenery (buildings, trees, static props) fresh from the world's data archives.
    5. `setLoadProgress <61..100>` + `loadKQ <objectName>.kq` for every remaining live persistent object in `KQPersistManager` (starting at index 2 after Terrain and Camera).
    6. `loadKQ WorldState.kq` — re-queues all pending timed `KQEvent`s.
    7. `setLoadProgress 100`.
40. **Why deleted/spawned objects, doors, chests, and coins come back accurately:**
    - **Deleted objects** (picked-up items, removed dead monsters) are no longer in `KQPersistManager` or `WorldInventory`, so they are absent from `newWorld.cs` and `WorldInventory.kq` and never respawn.
    - **Dynamically spawned loose items** (such as coins dropped by chests or killed monsters) live inside `WorldInventory.kq` (`KQWorldInventory`) as full `KQInventoryItem` objects with their exact world `TMat3F`, `quantity`, and `itemType`, so they reappear exactly where they dropped.
    - **Doors (`KQDoor`), chests (`KQChest`), and animated objects (`KQAnimObject`)** restore their exact `TMat3F` transform, `currAnimName`, `animPos`, `state` / `chestState`, `invItemName`, and `hidden` flags directly from their `<name>.kq` chunk.
    - **Camera (`KQCamera.kq`)** restores its exact position (`loc`), rotation (`cameraRotation`, `dir`), follow target (`followObjectName`, `pauseFollow`), offset (`cameraOffset`, `thirdcameraOffset`), and `isFirstPerson` mode.
    - **`world.cs <World> Begin` still runs** on the first drawn frame after `RestoreWorld` (just as on a portal return; see [world-change.md](world-change.md) rule 9), **not** skipped — first-time opening cutscenes do not replay because `WorldVar.cs` already restored their guard variables (such as `Daventry::girlhut::FirstTime = 0`).

## Numbers and units

| Quantity | Value | Unit / meaning |
|---|---|---|
| `PVOL` `voli` record size | `17` | bytes (`u32`, `u32`, `u32`, `u32`, `u8`) |
| `VBLK` alignment | `4` | bytes (bit 31 set on block size) |
| `PERS` / `DICT` alignment | `2` | bytes (bit 31 clear on block size) |
| Max `PERS` class name length | `128` | ASCII bytes (`u16` length, padded to even) |
| `TMat3F` binary size | `52` | bytes (`u32 flags` + `9 × f32` + `3 × f32`) |
| Inline `KQMover` binary size | `68` | bytes (`TMat3F` + `3 × i32` + `f32`) |
| `KQGameState` `PERS` chunk size | `44` | bytes (`GameState.kq`) |
| `KQCamMover` `PERS` chunk size | `502` | bytes (`KQCamera.kq` with `followObjectName="Connor"`) |
| `savedGameVersion` | `"1.000000"` | 8 ASCII bytes |

## Data used

- `MASK/save/save1.vol`–`save24.vol` (24 real saved games covering Daventry, Castle Daventry, Dead City, and Swamp; 9,686 binary `PERS` chunks across 28 distinct classes plus embedded ASCII kept-world `PVOL` archives and maps).
- `MASK/mask.cs` (`RestoreWorld`, `NewWorld`, `UnloadWorld`).

## Edge cases

- **Odd-length class names and strings:** Both `PERS` class names and binary `string` payloads are padded to an even byte length (`(len + 1) & ~1` for class names without null; `((strlen + 1) + 1) & ~1` for strings with null). A reader that forgets the 1-byte pad on odd class names (e.g., `KQGameState` length 11, `KQTerrain` length 9, `KQMap` length 5, `KQMusic` length 7) will desync by 1 byte immediately.
- **Empty vs null strings:** A null string pointer writes `u32(0)` (4 bytes). A non-null empty string `""` writes `u32(2)` followed by `0x00 0x00`.
- **Two-way compatibility shortcut:** Because `Mask.exe`'s `loadKQ` transparently accepts ASCII `.kq` INI format for any file whose first 4 bytes are not `"PERS"`/`"DICT"`, an implementation that **reads** both binary `PERS` and ASCII `.kq` INI can load every original `Mask.exe` save, and can **write** either binary `PERS` or ASCII `.kq` INI into `save<N>.vol` and have the original `Mask.exe` load it cleanly.

## Do not reproduce

- None identified for the file format itself; follow the exact `PVOL` and `PERS` framing for two-way compatibility with original saves.

## How to check

- Parse all 24 `MASK/save/save*.vol` archives with a `PERS` reader: all `9,686` binary `PERS` chunks across all 28 classes must parse to `0` leftover bytes (`save1.vol`: `GameState.kq` `nextNoName=439`, `difficultyFactor=0.66`; `worldState.kq` 2 `KQCueEvent`s; `Connor.kq` `stats="10 10 0 0 0"`, `currAnimName="Stop"`, `animPos=0.2667`; `Condoor.kq` `KQDoor` `state=2`, `currAnimName="closed"`).
- Inspect `save18.vol`: current world (`Swamp.kq`) chunks and `swmpMap.kq` are binary `PERS`, while embedded `CastleD.vol`, `Daventry.vol`, `DeadCity.vol` and `castMap.kq`, `davMap.kq`, `dedMap.kq` are ASCII `.kq` INI.

## Open questions

- None for `Q207`. Niche `KQState` subclasses for late-world puzzles not present in the 24 Daventry–Swamp saves (such as `KQStateConnerTile2`/`3`, `KQStateConnerScales`) follow the same `KQBase` + `fromState` + class fields pattern if ever needed.
