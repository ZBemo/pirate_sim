# Camera system

## The Problem
Right now with our system of generating frames and then sending them to the render thread, we are putting a 
disproportianate amount of work on the render thread. The whole point of having rendering and "work" in a seperate thread was to avoid having 
the work thread need to render, freeing up time for quicker responses. However, right now we are essentially having both the work & render thread 
render a frame, which not only does little to shift work off of the work thread, but is essentially doing twice the work for each frame. 
Although it would be a heavy time investment to get the architecture set up, having a camera system would do a lot to aleve this, 
as it would allow the rendering system to do nearly all of the rendering work, while the work thread simply needs to register some properties on start up.

## Architecting the camera system
**Everything below is an extremely abstract and unrefined idea for the camera architecture**

This might require integrating brevy ECS in order to make it feasible.

First, we need to instill the renderer with some idea of "phsyical space", so that it understands what the camera should and should not render at any 
given moment.

Next, We need a list of "renderables" with a position in physical space and some other information.

This may require a mutex.

On each render tick, we would lock renderables, iterate through nad render them if theyre "on screen".
