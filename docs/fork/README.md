# Fork layout (konashevich/QymCAD)

Fork operational notes that Cursor must see every turn live in the repository-root
`AGENTS.md` (fork section under the upstream paragraph). This directory explains
branch layout. None of that must appear in a pull request to `QymIs-Tech/QymCAD`.

## Remotes

| Remote   | URL                                      | Role                                      |
| -------- | ---------------------------------------- | ----------------------------------------- |
| `upstream` | `https://github.com/QymIs-Tech/QymCAD.git` | Parent project                            |
| `origin`   | `https://github.com/konashevich/QymCAD.git` | This fork                                 |

## Branches

| Branch              | Contents                                                                 | Goes upstream? |
| ------------------- | ------------------------------------------------------------------------ | -------------- |
| `upstream/main`     | Parent `main`                                                            | n/a            |
| `local/ops`         | Fork docs: expanded root `AGENTS.md`, `docs/fork/`, Cursor fork rule, ignore for `*.local.md` scratch | No             |
| `feat/qymcad-drive` | Product work for the headless / API session (`qymcad-drive`, i18n, lock)  | Yes (when ready) |
| `main` (this fork)  | Working integration: `local/ops` + current feature branch(es)            | No as a whole  |

Daily work on this machine uses fork `main` so agents see both the feature code and
the fork notes. A pull request to upstream is opened from `feat/qymcad-drive` (or a
later feature branch), never from fork `main` and never from `local/ops`.

## Rules that keep the PR clean

1. On `local/ops` / fork `main`, root `AGENTS.md` may carry a fork section so Cursor
   auto-loads it. On `feat/qymcad-drive`, root `AGENTS.md` must stay identical to
   upstream (no fork section).
2. Do not put install scripts, Rockchip notes, or MCP private scratch into the feature
   branch.
3. Personal scratch that must not sync even to the fork stays untracked:
   `CLAUDE.local.md`, `agent-api.local.md` (and anything matching `*.local.md`).
4. Before opening a PR: `git fetch upstream` and rebase `feat/qymcad-drive` onto
   `upstream/main`. Confirm `git diff upstream/main...HEAD` lists only product files
   and does not change root `AGENTS.md`.

## Installed app vs this tree

The binary launched as `qymcad` is built from
`/mnt/merged_ssd/app_data/qymcad/QymCAD` by `qymcad-update`. That clone tracks
upstream only. Feature experiments in this AppDev tree do not change the installed
app until someone installs a build from here on purpose.
