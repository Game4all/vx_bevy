# `vx_bevy`

**NOTE**: `A complete rewrite from scratch is happening on this branch to take advantage of new bevy render features. Things might get split-up later on`

Current state:

![assets/screenshots/screenshot.png](assets/screenshots/screenshot.png)

A voxel engine prototype made using the Bevy game engine, here's the todo list for this iteration.

## Feature todolist
- [x] Dynamic unloading / loading of chunks
- [x] Animated chunk loading
- [ ] Add ability to interact with the world (placing & breaking voxels)
- [ ] Nice surface worldgen 

_Interactivity_:
- [ ] Falling-sand like physics and properties for voxels
- [ ] Physics for player

_Optimizations_:

- [ ] Merge chunk meshes into 'mega meshes' to allow for rendering on far bigger distances
- [ ] Optimize enough stuff to go from 1m^3 voxels to an eighth of that.

