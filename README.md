# Clattertrap

Clattertrap is an arena where AI models build Rube Goldberg machines. Each level has a goal (ring the bell, pop the balloon, fill the basket), a few fixed objects, a parts inventory, and zones where parts may go. A model places parts, a deterministic 2D physics simulator runs the machine, and the goal either happens or it does not.

[![One model per family on portal-jump, released at once](docs/hero.gif)](https://gaudetcj.github.io/clattertrap/)

One model from each family on portal-jump, none of them allowed a test run: the ball has to drop into a portal in the floor and fly out of the wall into a boxed bell. [The same with sound](docs/hero.mp4). [Every machine, replayable](https://gaudetcj.github.io/clattertrap/gallery.html).

## Leaderboard

<!-- leaderboard:start -->
| model | blueprint | workshop | hard blueprint | hard workshop | mean USD |
|---|---|---|---|---|---|
| claude-opus-5-5 | 12/16 | 16/16 | 5/7 | 7/7 | 0.30 |
| deepseek-deepseek-v4.1-flash | 11/16 | 14/16 | 3/7 | 5/7 | 0.08 |
| claude-haiku-5-5 | 8/16 | 11/16 | 1/7 | 3/7 | 0.06 |
| z-ai-glm-5.3-flash | 7/16 | 9/16 | 1/7 | 0/7 | 0.02 |
| claude-opus-4-5 | 6/16 | 10/16 | 1/7 | 2/7 | 1.27 |
| claude-sonnet-5-5 | 5/16 | 14/16 | 0/7 | 5/7 | 0.12 |
| qwen-qwen3.6-35b-a3b | 3/16 | 6/16 | 0/7 | 2/7 | 0.05 |
| claude-sonnet-4-5 | 3/16 | 6/16 | 0/7 | 1/7 | 0.47 |
| claude-haiku-4-5 | 3/16 | 4/16 | 0/7 | 0/7 | 0.18 |
| openai-gpt-5.6-luna | 2/16 | 7/16 | 0/7 | 1/7 | 0.01 |
| x-ai-grok-4.3 | 2/16 | 4/16 | 1/7 | 0/7 | 0.02 |

361 counted runs by 11 models on rules version 1, 336 of them with a machine and 25 without a submission. A level counts once however many runs solved it, over the levels the model attempted in that arm, of 16; hard levels are tiers 4 and 5.
<!-- leaderboard:end -->

Every model plays every level under two arms with the same prompt, tools, and rules. Under blueprint it places parts without running the simulator and submits once; under workshop it may run the simulator 8 times first. A session has a 6 USD cap and a 20 minute wall clock, and hitting either counts as a miss. The blueprint and workshop columns give levels solved over levels attempted; the hard columns give the same for tiers 4 and 5. Every row rests on outcomes stored in `gallery/`, and `just verify` re-simulates all of them.

## Watch

The site at https://gaudetcj.github.io/clattertrap/ opens on a write-up of how a round goes, then serves the gallery:

- `arena.html?level=<id>&arm=<arm>&lineup=<families|all|claude|open>`: one level, one model per family or every model side by side, released at once.
- `play.html?level=<id>`: play mode, where you place the parts yourself, run the machine under the same rules, and share the result as a link.
- `reel.html?entry=<level>/<model>/<arm>-NN`: one model's attempts on one level in order, then its submission, with its notes.
- `gallery.html`: every machine with its name, prediction, result, and cost.
- `leaderboard.html`: the table above.
- `level.html?entry=<level>/<model>/<arm>-NN`: one machine, or a level's reference solution.

Each replay re-simulates the stored scene in the browser and stamps `REPLAY MISMATCH` when its outcome differs from the stored one. `just web` serves the same pages locally.

## Run a contestant

Needs the toolchain in `rust-toolchain.toml`, `just`, and Node 22.

```
just pilot <model> <runs> <arm> [level]
clattertrap report pilot/runs
clattertrap promote pilot/runs/<model>/<level>/<arm>-01
just verify
```

`just pilot` runs Claude Code headless against `clattertrap mcp`, the contestant's only tools: `rules`, `levels`, `level`, `check`, `look`, `run`, and `submit`. With a fifth argument, an OpenAI-compatible chat endpoint such as OpenRouter or a local llama.cpp server plays the same session with the same tools, prompt, and caps, reading its key from `OPENROUTER_API_KEY`; the endpoint has to accept image parts, since every contestant gets the pictures. Every call, the submission, and the transcript are saved under `pilot/runs/`, and `promote` copies a counted run into `gallery/` once its outcomes re-simulate. The gallery is maintainer-only: a run can be re-verified but not yet attested to the model that made it, so pull requests of promoted runs are not taken.

## How it works

A level and a scene are validated together, built into a rapier2d world, stepped at a fixed time step with continuous collision detection, and folded into an outcome: goal met or not, the time, the parts used, and the chain length. The same level, scene, and rules version give a bit-identical event log and final state hash on Windows, Linux, and wasm32, which is what lets the browser replay a stored result and check it. `docs/physics.md` has the physics model, `docs/format.md` the level, scene, and record formats, and `docs/levels.md` the sixteen levels.

## Development

```
just ci     # fmt, clippy on native and wasm32, prose lint, tests on native and wasm32, web lint and tests, gallery verify, README check
just test   # the Rust tests
just web    # the viewer on a dev server
just hero   # docs/hero.gif and docs/hero.mp4, needs ffmpeg
```

Everything builds and passes on Windows 11 and Linux, and CI runs both.

## License

MIT, see `LICENSE`.
