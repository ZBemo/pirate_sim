This is a todo list for the entire project. Long term goals are entirely out of order, and short term is roughly in order


# short term: 
 - [x] change InputPacket & RenderPacket to better names
   - RenderTick & RenderUpdate 
   - WorkPacket (from work) & RenderPacket (from renderer) or vice versa
 - [ ] fully architect render.rs
 - [ ] make worldgen module prettier, probably bring some things out of worldgen/mod.rs
 - [ ] sort out render and work thread (see src/main:163)
 - [ ] give worldgen more control over rendering
 - [ ] finish erosion (see src/worldgen/mod.rs:207)

 # long term:
 - [ ] port to bevy engine. bracket-lib seems pretty much dead, and we can still use it for randomness
    - ECS will probably be necessary for rendering improvements, and will just make gameplay implementation way easier
    - will be fun
 - [ ] menu system
 - [ ] saving worlds
 - [ ] rpg gameplay
 - [ ] history generation
 - [ ] fix visibility \(pub\(crate), pub\(self), etc for stricter visibility, less leaking of API)
 - [ ] camera system with mutex, or something similar \(see [planning/camera.md](./planning/camera.md))
 
