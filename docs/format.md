# Level and scene formats

The two JSON documents in Clattertrap: a level, written by hand and read by everyone, and a scene, written by a contestant and checked against its level. Both are plain serde types in `clattertrap-core`, and the JSON Schemas come from those types: `clattertrap rules --schema` prints the scene schema, `clattertrap rules --level-schema` the level schema. Every field name carries its unit: `x_cm`, `angle_deg`, `time_limit_s`. Unknown fields are rejected.

## Level

A level lives in `levels/<id>/level.json`. Its reference solution, a plain scene, lives beside it in `reference.json` and never reaches a contestant through any command.

| field | type | meaning |
|---|---|---|
| `id` | string | the directory name, and what a scene's `level` field must say |
| `title` | string | short title for viewers |
| `tier` | 1 to 5 | difficulty, from one part to several mechanisms in a chain |
| `description` | string | the layout in prose: where things are and what moves at the start |
| `rules_version` | integer | the rules the level was authored for; `clattertrap` refuses others |
| `world` | `{ width_cm, height_cm }` | the world rectangle, origin bottom-left, y up |
| `time_limit_s` | number | run length cap |
| `fixed` | list of placed objects | what the level places, goal objects included |
| `zones` | list of `{ x_cm, y_cm, width_cm, height_cm }` | where a contestant may put part centres; `x_cm, y_cm` is the bottom-left corner |
| `inventory` | `{ kind: count }` | how many of each part kind a contestant may place; missing kinds are 0 |
| `goal` | object | see below |
| `showcase` | bool, default false | whether chain length is scored too |

A goal is `{ "kind": "ring" | "press" | "fill" | "pop", "target": "<fixed id>" }`. `fill` adds `count`, the number of objects that must rest inside the basket. Any goal may add `via`, a list of part ids that must be touched in that order before the goal counts. The target must be a fixed object of the matching kind: a bell for `ring`, a button for `press`, a basket for `fill`, a balloon for `pop`.

## Scene

```json
{ "level": "bell-drop", "parts": [
  { "id": "b1", "kind": "beam", "x_cm": 120, "y_cm": 210, "length_cm": 120, "angle_deg": -15 }
] }
```

Each part has a unique `id`, a `kind`, and the fields of that kind. Positions are the part's centre in centimetres except where the table says otherwise. Angles are degrees, counterclockwise positive. Ranges come from `rules.rs` and are what `check` enforces.

| kind | fields |
|---|---|
| `beam` | `x_cm`, `y_cm`, `length_cm` (20 to 150), `angle_deg` |
| `funnel` | `x_cm`, `y_cm` (the throat's centre), `mouth_cm` (30 to 150), `throat_cm` (14 to the mouth), `height_cm` (20 to 80) |
| `ball` | `x_cm`, `y_cm`, `material` (`steel`, `wood`, `rubber`) |
| `domino` | `x_cm`, `y_cm`, `angle_deg` (default 0) |
| `crate` | `x_cm`, `y_cm`, `side_cm` (10 to 40) |
| `seesaw` | `x_cm`, `y_cm` (the pivot), `length_cm` (40 to 150), `angle_deg` (default 0) |
| `pendulum` | `x_cm`, `y_cm` (the pin), `length_cm` (20 to 120), `angle_deg` (default 0, from straight down) |
| `spring_pad` | `x_cm`, `y_cm`, `angle_deg` (default 0) |
| `bucket` | `x_cm`, `y_cm` (centre of the floor's underside), `width_cm` and `height_cm` (15 to 60) |
| `balloon` | `x_cm`, `y_cm` |
| `pin` | `x_cm`, `y_cm` (the base), `angle_deg` (default -90, pointing down) |
| `rope` | `a`, `b` (ids of two solid parts), `max_length_cm` (10 to 200) |

The goal objects `bell` (`rope_cm`, 10 to 100), `button` (`travel_cm`, 1 to 10), and `basket` (`inner_width_cm`, `inner_height_cm`, 15 to 60) use the same shape but only a level may place them. A bell's `y_cm` is its centre, a button's the centre of its cap's resting underside, a basket's the centre of its floor's underside.

The parser is stricter than the schema in one respect: a part with a field its kind does not have is rejected, while the schema lists each kind's own fields beside `id` and does not forbid extras.

## Validation messages

`check`, `look`, `run`, and the viewer all report problems the same way, one per line, every problem at once:

```
<path>: <what is wrong>; <one concrete fix>
```

The path is `level`, `scene`, `parts[i] (kind id)`, `fixed[i] (kind id)`, `goal.via[i]`, or the JSON path of a parse error. The checks, in order:

| check | example |
|---|---|
| JSON parses | `scene: key must be a string at line 1 column 25; write what the format asks for there` |
| level id matches | `level: is "other" but this level is "bell-drop"; set level to "bell-drop"` |
| tier in range | `level: tier 6 is outside 1..5; set tier within that range` |
| ids unique, not `floor` | `parts[1] (beam far): id "far" is already used; give this part a new id` |
| no goal objects in a scene | `parts[0] (bell b): bell is a goal object only the level may place; remove it` |
| inventory | `parts[2] (beam b2): the inventory allows 2 beam parts and this is number 3; remove one` |
| centre inside a zone | `parts[0] (ball k1): centre (10, 290) is outside every build zone; move it inside x 30..340, y 0..300` |
| lengths in range, numbers finite | `parts[1] (beam far): length_cm 500 is outside 20..150; set length_cm within that range` |
| rope ends and waypoints exist | `parts[0] (rope r): a = "nope" names no part; link two placed parts` |
| goal target is a fixed object of the right kind | `goal: target "bell" names no fixed bell; add a bell with that id to fixed or change the target` |
| nothing overlaps at the start | `parts[0] (domino low): overlaps floor by 5.0 cm at the start; move it up to y_cm = 10.0` |

The overlap check runs only once everything else passes. Two fixed objects that overlap each other are not reported, since the engine never moves them.

## Outcome

`clattertrap run` prints the outcome as JSON:

```json
{
  "goal_met": true, "end": "goal_met", "time_s": 1.9250001, "steps": 231,
  "parts_used": 2, "chain_length": 2,
  "event_hash": "41a3e9905cb8f870", "state_hash": "bbf5aaffac928e50", "rules_version": 1
}
```

`end` is `goal_met`, `time_limit`, or `all_asleep`. The two hashes are 16 hex digits each and are the same on every platform for the same level, scene, and rules version. With `--trace`, each event comes first, one per line: the time in seconds, the kind (`contact`, `via`, `goal_met`, `at_rest`, `left_world`), and the parts by name.

## Pilot records

`clattertrap arena` writes one directory per session under `pilot/runs/<model>/<level>/<arm>-NN`, numbered from 01. The arms are `blueprint`, with no simulator runs, and `workshop`, with 8. Every session has the same prompt (`pilot/prompt.txt` with the level id and the run budget filled in), the same 6 USD cap, and the same 20 minute wall clock.

| file | written by | holds |
|---|---|---|
| `mcp.json` | the runner | the MCP config for this session: this binary serving `mcp --level <id> --arm <arm> --out <dir>` |
| `transcript.jsonl` | the runner | Claude Code's stream-json output, one record per line |
| `stderr.txt` | the runner | whatever Claude Code printed on stderr |
| `calls/NNNN-<tool>.json` | the MCP server | every `check`, `look`, `run`, and `submit` call: `seq`, `tool`, the `scene` as sent, and either `problems` or the `outcome` |
| `submission.json` | the MCP server | the one submission: `level`, `arm`, `name`, `prediction`, `scene`, `outcome` |
| `summary.json` | the runner | `model`, `level`, `arm`, `status`, `started_unix_s`, `duration_s`, `cost_usd`, `turns`, `tool_calls`, `foreign_tools`, `result_subtype`, `exit_code`, `error` |

`status` is one of:

| status | meaning | counts |
|---|---|---|
| `submitted` | the contestant submitted a machine | yes |
| `no_submission` | the session ended on its own without a submission | yes, as a miss |
| `over_budget` | the dollar cap stopped the session first | yes, as a miss |
| `timed_out` | the wall clock stopped the session first | yes, as a miss |
| `aborted` | an error, a usage limit, or an MCP server that never connected | no; run it again |
| `flagged` | a tool call outside the clattertrap MCP tools | no |

`clattertrap report <dir>` prints, per model and arm, the levels solved (a level counts once however many runs solved it), the hard levels solved (tiers 4 and 5), the runs counted, the aborted and flagged runs, and the mean cost and time. A second table gives, per level, the runs, the runs that solved it, the attempts (`run` calls, accepted or not), the scene calls accepted and rejected by validation, and the submissions. `--csv` prints one row per run, every run included. `--sample N --seed S` prints N counted submissions to open on the level page. In a checkout with pilot runs, `just web` lists every submission under `pilot/runs/` in a picker on that page, and `?run=<model>/<level>/<arm>-NN` opens one directly; the page also plays any dropped scene or submission file.

## Exit codes

0 when the command did its job, including a run whose goal was missed. 1 when `check`, `look`, or `run` rejected the scene; the problems are on stdout. 2 for a usage error, an unknown level, or a file that could not be read; the reason is on stderr.
