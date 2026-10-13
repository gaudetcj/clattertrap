# Physics model, rules v1

This describes what `clattertrap-core` simulates today. Every number lives in `rules.rs` as `Rules::V1`; this page explains them. The engine is `rapier2d` 0.36.1 with the `enhanced-determinism` feature, no `simd8`, no `parallel`.

## Units and frame

- Levels and scenes use centimetres and degrees in their JSON: `x_cm`, `length_cm`, `angle_deg`. The world converts to metres and radians once, when it is built.
- Origin bottom-left, y up. Angles are counterclockwise positive. A beam at `angle_deg: -15` slopes down to the right.
- Gravity is 9.81 m/s² downward.
- The time step is 1/120 s. Simulated time is `step * dt_s`, never an accumulated sum. A 20 s level is 2400 steps.
- Continuous collision detection is on for every dynamic body, with up to 4 substeps.
- The floor is a fixed slab along the bottom edge. There are no walls and no ceiling. The floor extends a quarter metre past each side so a part that rolls off the edge drops out of sight.

## Materials

Two-dimensional density is mass per square metre, so a 6 cm steel ball weighs about 1 kg and a wood ball about 0.15 kg.

| material | density kg/m² | restitution | friction |
|---|---|---|---|
| steel | 88 | 0.2 | 0.2 |
| wood | 13 | 0.3 | 0.5 |
| rubber | 18 | 0.8 | 0.9 |

Restitution between two bodies is the larger of the two, so a rubber ball bounces on a dead floor. Friction is the average, except balloons, which take the smaller value.

## Parts

| kind | bodies | notes |
|---|---|---|
| beam | fixed box, 4 cm thick | wood friction, no bounce |
| funnel | two fixed walls, 2 cm thick | from a mouth down to a throat at least 14 cm wide |
| ball | dynamic disc, radius 6 cm | one of the three materials |
| domino | dynamic box, 4 by 20 cm | wood |
| crate | dynamic box | wood |
| seesaw | dynamic plank on a revolute joint to a fixed pivot at its centre | wood, 4 cm thick |
| pendulum | steel disc, radius 5 cm, on a revolute joint to a fixed pin | the rope is rigid |
| spring pad | fixed plate, 30 by 4 cm | see spring pad below |
| bucket | one dynamic body: a floor and two walls, 2 cm thick | wood |
| balloon | dynamic disc, radius 10 cm | see balloon below |
| pin | fixed needle, 1 by 16 cm, base at the given point, pointing along `angle_deg` | pops balloons |
| rope | a rope joint between the centres of two parts | no body of its own |

Goal objects, which only a level may place:

| kind | bodies | notes |
|---|---|---|
| bell | brass disc, radius 10 cm, hung by a revolute joint from a fixed anchor `rope_cm` above it | rings on impulse, see below |
| button | wood cap, 12 by 3 cm, on a sprung prismatic joint above a fixed base | pressed at full travel, see below |
| basket | one fixed body: a floor and two walls, 2 cm thick | filled by resting objects, see below |

### Rolling

A disc rolling without slipping down a slope of angle θ accelerates at `g sin θ / (1 + I / m r²)`, and a uniform disc has `I = m r² / 2`, so `a = (2/3) g sin θ`. The incline test checks this to within 10 percent.

### Pendulum

`angle_deg` is measured from straight down, counterclockwise positive, so the weight starts at `pin + L (sin θ, -cos θ)`. The small-angle period is `2π sqrt(L / g)`.

### Spring pad

When a dynamic body starts touching a pad, it receives an impulse of 0.6 N·s along the pad's up normal, once per contact start. Light parts fly higher than heavy ones.

### Balloon

A balloon has density 1 kg/m², a gravity scale of -0.5 so it rises, and linear damping 3.3, which caps its rise at about 1.5 m/s. It slides along the underside of anything above it toward the higher end. Touching a pin removes it. A balloon that drifts out of the world is removed too, but that is not a pop.

### Bell

The bell collider reports contact forces above `bell_min_impulse_ns / dt_s`. The simulator converts the reported force back to an impulse and rings when it reaches 0.2 N·s. A wood ball at 1 m/s or a steel ball at 0.2 m/s is enough.

### Button

The cap rides a prismatic joint along y with limits `[-travel, 0]` and a force-based spring of 50 N/m with damping 2. The button counts as pressed when the cap's underside is within 2 mm of full travel. A resting wood ball presses a 3 cm button; the cap alone sags about 1 cm.

### Basket

An object counts once its centre is inside the inner rectangle and its speed is below 0.02 m/s. The goal fires when the count reaches the level's `count`.

## Goals and waypoints

A goal is one of `ring`, `press`, `fill`, or `pop` and names a fixed object of the matching kind. `via` lists part ids that must be touched in order first; each touch emits a `Via` event, and the goal is ignored until the list is done.

## Events

Each step's events are sorted by kind, then by part id, and duplicates are dropped.

- `Contact { a, b }`: two parts started touching; `a` is the smaller id.
- `Via(part)`: a waypoint was touched in order.
- `GoalMet`: the goal fired; the run ends on this step.
- `AtRest(part)`: a dynamic part fell asleep.
- `LeftWorld(part)`: a dynamic part's centre went more than 0.5 m outside the level rectangle; its bodies were removed.

Part ids index the validated scene's name table: fixed objects first, then the contestant's parts, then the floor.

## Run end

A run ends on the first of: the goal is met, the time limit passes, or every dynamic body is asleep. The outcome records which, the time, the parts placed, and the chain length: the number of distinct parts whose speed exceeded 0.05 m/s before the end.

## Hashes

Two FNV-1a 64 hashes identify a run. The event hash covers every event's step and bytes. The state hash covers, in body insertion order, the bit patterns of each body's position, angle, linear and angular velocity, and its sleep flag. A removed body hashes as a single marker byte. The same level, scene, and rules give the same hashes on Windows, Linux, and wasm32. In a serialized outcome both hashes are strings of 16 hex digits, since JSON numbers cannot hold a u64 exactly.

## Viewer

The browser page re-simulates a scene in the same core compiled to wasm32, so the hashes it shows are the ones the native simulator produces. It never decides an outcome itself. Bodies are drawn from each slot's `Visual` and pose; a pendulum rod, a bell rope, and a rope part are drawn as lines between their bodies. Sound is synthesized from the event log: a contact's timbre comes from the two parts' looks and its volume from the faster part's speed, a pin meeting a balloon pops, and the goal event voices the bell, button, or basket.
