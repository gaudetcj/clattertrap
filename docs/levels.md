# Levels

Fifteen levels live under `levels/`, one directory each with `level.json` and a hidden `reference.json`. Every reference solution meets its goal and every empty scene misses; both are golden-hash tests that run on Windows, Linux, and wasm32. `clattertrap levels` lists them and `clattertrap level <id>` prints one in words and JSON.

Tiers run from 1 to 5 and say how much a solution needs: one part placed well, a couple of ramps, a thrown object, a precise flight, several mechanisms in a chain. The pilot counts tiers 4 and 5 as hard.

| id | tier | goal | inventory | what it takes |
|---|---|---|---|---|
| bell-drop | 1 | ring | beam 4, seesaw 1, domino 6 | guide a falling steel ball to a bell hung low on the right |
| pendulum-bell | 1 | ring | pendulum 1 | hang one pendulum so its weight swings into the bell |
| balloon-pin | 2 | pop | beam 4, ball 1 | steer a rising balloon along beams to a pin near the top |
| basket-fill | 2 | fill 2 | beam 5, seesaw 1 | bring two of three falling wood balls to rest in a basket across the world |
| button-drop | 2 | press | beam 3, domino 8, funnel 1 | land a steel ball on a floor button and keep it there |
| domino-run | 2, showcase | fill 2 | domino 10, beam 1 | a wood ball rolls onto a shelf; a row of dominoes must topple two objects off the end into a basket |
| funnel-basket | 3 | fill 3 | funnel 1, beam 3 | three balls fall far apart and all must rest in one basket without jamming |
| post-first | 3 | ring, via post | beam 4 | the ball must touch a fixed ledge on its way to the bell |
| rope-swing | 3 | ring | rope 1, beam 2 | tie the falling ball to a fixed hook so it swings in an arc into the bell |
| seesaw-launch | 4 | ring | seesaw 1, domino 1, beam 2 | the bell sits in a box open only below; a seesaw must throw something up into it |
| spring-launch | 4 | press | spring_pad 1, beam 3 | a rolling wood ball must be kicked by a spring pad onto a high ledge and drop into a button slot |
| grand-tour | 5, showcase | ring, via post | beam 6, domino 8, seesaw 1, ball 2, funnel 1, spring_pad 1, crate 2, pendulum 1, bucket 1, rope 1 | a ramp to the post, a drop onto a long seesaw left of a boxed bell, and its far arm swinging up through the open bottom, in that order |
| uphill-relay | 5, showcase | ring, via a, b, c | beam 6, seesaw 1, ball 2, domino 4, crate 1, spring_pad 1, funnel 1 | the ball touches a high post, a post by the floor, and a post 70 cm back up, in that order; the last one takes a launch, since nothing rolling off the low post can reach it |
| wake-the-pendulum | 5, showcase | ring, via pend, ball1, post | beam 5, domino 4, ball 1, crate 1, funnel 1, seesaw 1 | the ball sets a hanging pendulum swinging, whose weight knocks a wood ball off a ledge onto a post, and that ball drops onto a bell |
| return-trip | 5, showcase | fill 3, via a, w1, d | beam 6, funnel 1, seesaw 1, bucket 1, crate 1, domino 4 | the ball crosses the world to knock two wood balls off a shelf, and all three have to come back past a low post into a basket on the left |

Every level is 400 by 300 cm with a 20 s limit. Fixed objects are placed exactly at rest, so a waypoint that stands on the floor is not touched until something moving arrives. The three levels with three waypoints force a chain of at least four stages rather than scoring it; a goal that is satisfied before its last waypoint is touched does not count.

The inventories are deliberately larger than the reference solutions use. A contestant may build a longer machine than it needs, and on showcase levels the chain length counts.
