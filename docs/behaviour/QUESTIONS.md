# Questions for the study session

The implementation session (B) adds questions here when a behaviour note is
unclear, incomplete or seems wrong. The study session (A) answers them in
the relevant note and marks them answered here.

| # | Note | Question | Asked by | Status |
| --- | --- | --- | --- | --- |
| Q1 | none yet (models, `.kqs`) | How are model transforms applied? Each mesh node and sequence has a 3×3 matrix + translation. We use: point as a row vector × matrix + translation, mesh node first, then sequence. Data hint (`kq8 shape-verify`): parts of multi-part models meet more often this way (4,891 touching vertices vs 2,761 transposed; 245 models clearly better vs 7), but that is not proof. Please confirm (a) row or column vector, (b) node then sequence or the other order, (c) whether the flag bits (1 rotation, 2 translation, 4 scale) change how the matrix is used. | B, M1 step 1, 2026-10-11 | open |
| Q2 | none yet (models) | Which side of a triangle is the front (corner order, seen from which side), are back faces hidden, and is the game world right- or left-handed (do we need a mirror when converting to a Y-up renderer)? Measured: Z is up. We draw both sides for now. | B, M1 step 1, 2026-10-11 | open |
| Q3 | none yet (textures) | Which palette colours model textures? Chunked textures have none of their own. Measured: `resource/daventry.ppl` (same colours as `8bit/daventry.pal`) gives the smoothest pictures, `8bit/daventr1.pal` worse. Windows-BMP textures carry a colour table, and for 4 Daventry textures every used colour equals `daventry.ppl`. Are the first/last 8 system colours replaced for textures too? | B, M1 step 1, 2026-10-11 | open |
| Q4 | none yet (textures) | Where is a texture name looked up? The 1,805 models in `GAME/Common` name textures that exist only inside world archives (none in `GAME/`); many names exist in several worlds. What is the search order, and what happens when a name is missing everywhere (29 names, e.g. `grani13.bmp`, `marbl11.bmp`, `stone04.bmp`)? | B, M1 step 1, 2026-10-11 | open |
| Q5 | none yet (materials) | What do the unknown material flag bits 0x1000, 0x10000 and 0xF000 do? Seen: 0x403 ×11,221, 0x103 ×1,538, 0x10103 ×142, 0x203 ×75, 0x1403 ×20, 0x10403 ×1, 0x402 ×1, 0xF000 ×31, 0x0 ×192. Is one of them transparency, and if so which colour or palette index is see-through? | B, M1 step 1, 2026-10-11 | open |
