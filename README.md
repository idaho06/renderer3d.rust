# 3D Renderer in Rust
I still did not finish the awesome Pikuma's course [3D Computer Graphics Programming](https://pikuma.com/courses/learn-3d-computer-graphics-programming) and I just needed to test my Rust skills. This course helped me to understand the math and high level logic, but the code is directly inpired by the book [Computer Graphics from Scratch](https://gabrielgambetta.com/computer-graphics-from-scratch/) by Gabriel Gambetta. I found the math explanations much harder to follow, but the code is easy to read.

So here we are, creating another software rasterizer just for fun.

I managed to render textured triangles in 3D, loaded from an obj and a png files.

Stuff I would like to code:

- Optimize the code as much as I can. I'm using Optik to profile and following the recomendations from the [The Rust Performance Book](https://nnethercote.github.io/perf-book/). I would like to test Cargo's flamegraph and Valgrind
- ~~Simple textured triangle lightning~~
- Gouraud shading? Phong shading?
- Mesh clipping against the view fustrum
- Triangle clipping using Homogeneous coordinate clipping
- Simple scene management. Sets of static meshes, actors. Scene changes. Transitions?
- Vertex shaders to do fancy animations.
- Use a music library to play tracker music (mod, xm, s3m, it)
- Create a cool old-school demo and maybe... maybe... send it to a demo compo?