# laser potato design doc

laser potato is a stephen's sausage roll (3d sokobon with novel mechanics) inspired puzzle game centered around reflecting lasers with mirrors to accomplish some goal.

## game units

each puzzle is dilieated in the game, moves can be undo (z) and the entire puzzle can be reset (r)

each puzzle exists on a 3d isometric grid, we use RHS coordinates:

```
 (up)
 +Z
 ^  ^ +Y (forward)
 | /
 |/
 +-----> +X (right)
```

the grid is allowed to extend indefinitely in any direction + or -

## player character

the player controls a single character with arrow keys

there is also a single interaction button (space) and a "wait" button (a)

movement is turn 90 degree plus step 1 unit forward or backwards

under certain conditions, strafing movement is used instead of turning

inputing movement is only a request and may not actually result in movement, but even no-movement inputs may have other effects so they are still counted as an action on the stack.


## blocks

the game state is merely blocks on a grid, all blocks have the following common properties

- name
- position on grid
- rotation (1 of 24 possible rotations)
- movement priority


and each block may then have its own states


## state transition

states are split into "turns" which one state transforming into the next based on a set of rules each turn

state updates are fully reversible in order to support undo

### the 2 phase split state/block tranistion loop

we provide the option of splitting position and non position state transitions into 2 phases

we can optionally combine them to see which one presents more interesting behavior

the algorithm is as follows

at each turn, blocks may indicate they want to be in a new state/position/rotation (based on prev state and user input)

#### the split algorithm

for simplicity, position refers to position/rotation, and state refers to non position/rotation state

1. add all ojbects with move intentions onto the movement queue in order of priority, resolving by block position if there is a priority tie
2. process blocks on the queue
    - move the block, push any state changes onto the queue,marking it as being caused by this block move
    - based on new block position, compute downstream movements and push onto the movement movement queue, marking it as being caused by the block above
        - state are not possible here, but if ther were you would also push them onto the state queue
    - if a movement is illegal
        - don't move the block
        - if it was caused by some block, roll back the queue to that block and treat that move as illegal (don't forget to roll back the state queue as well)
    - repeat until the queue is empty
3. process locks on the state queue
    - same as step 2 but with state and position swapped
    - if any state change in the initial queue is illegal, you will roll back to previous step, marking that move as illegal
4. repeat until all queues are empty


#### the combined algorithm
1. add all bolkcs with state intentions onto the queue in order of priority, resolving by coordinates if there is a priority tie
2. process blocks on the queue
    - update the block
    - based on new block state, compute downstream block state changes, marking it as being caused by the block above
    - if a state change is illegal
        - revert the state change for that block
        - if it was caused by some block, roll back the queue to that block and treat that move as illegal (don't forget to roll back the state queue as well)
    - repeat until the queue is empty


### block state transition

after all movement is resolved, we then resolve states, this is very unique to each block type, and we will document the rules as they come along.

note that states may trigger other states which may put things back into some invalid state, so a similar queue like the one above may be needed, or perhaps we can comibne state and movement into 1 queue?

#### laser transition

a laser source is a block and a direction, lasers do take up space in the grid but are state only, not blocks!

- from the source, walk in the emit direction
- if a mirror block is hit, reflect the laser
- if a splitter block face is hit, the splitter absorbs new laser sources and becomes a new laser source
- if a block is hit, indicate to it that it's been hit, which may cause that block to change state


## random ideas to experiment with

- tile spinner (turns block on top of it once each turn or something)
- fused blocks
- would be cool to have "1 shot" triggers, rather than continuous triggers
	- colud have 1 time use unpowered sources
	- could have pulsed lasers so you need to hit a button to pulse the laser once
- perhaps laser should have both continuous low power, high power, and high power pulse modes
- maybe there is magnet mode where blokcs stick to you/each other, you can disengage by strafing the side fo block against wall)
- different color lasers
- laser combiners and mechanics that rely on lasers combining in general
- fused blocks
- 3d axis D:
	- the thing tha makes this tricky is that it's hard to bounce lasers straight up back into a XY plane, you need ot use fused blocks for this OR do the anti gravity thing
	- anti gravity flip (so cieling becomes ground and ground becomes cieling, maybe blocks stay where they last were, like english country tune logic)
	- rolling mirrors, mirrors that roll as you push them
- subgrid lasers, so each block can actually have 1-3 lasers, with one in the midle and 2 on the outside, and their ordering changes as you reflect it so it makes it more interesting, and you can split them etc
- rainbow lasers, similar to subgrid laser but it's one big wide laser
- destructible blocks 
- bombs that destroy stuff (maybe even indestructible block)