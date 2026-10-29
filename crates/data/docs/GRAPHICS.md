## `engine.rs`

`engine.rs` was derived from the game, and is made to replicate it. Some behaviors seen in the game I have found are unneeded for this engine, or would actively add bloat or add failure points an API should not have. See below.

**Divide by Zero:** A model whose unit columns are zero, or two keyframes inside a polynomial run that land on the same frame. Both cause the game to divide by zero and take the fault. I am not reproducing a crash. I leave the value alone in the first case and skip the term in the second. There is no correct result to replicate in this scenario, because the game just dies.

**Mirror Sprite Flag:** The game has a flag that negates the root's horizontal scale and the rotation angle. That is a faction-based flag, and it belongs to whatever places the entity in the world, so `resolve_frame` neither takes it nor applies it. A caller that wants a mirrored entity mirrors its own camera.

 **Single Texture Sizing:** A sheet with a flag set is sized from the texture instead of a cutout. The game makes the flag to false unconditionally on every spritesheet loading path, so anything built from a `.png` and an `.imgcut` always comes back with it clear.
