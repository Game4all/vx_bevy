# `vx_bevy`

A minecraft-esque voxel engine rendering prototype made using the Bevy game engine.

Chunk are rendered using a triangle mesh per chunk. Chunks are greedily meshed.

Performance is okayish (~100fps on a 1060 + 8th gen intel on release mode) with default render distance (16 chunks) altough mesh stitching could allow this to go even higher up.


Also don't go under the world.

## Screenshots

![assets/screenshots/screenshot.png](assets/screenshots/screenshot.png)
![assets/screenshots/clip.gif](assets/screenshots/clip.gif)

## Acknowledgments

This uses the awesome [block-mesh](https://github.com/bonsairobo/block-mesh-rs) crate which handles greedy meshing.