# Agents

The rules for any coding agent working in this repository are in [CLAUDE.md](CLAUDE.md): how comments,
interface strings and tests are written, how the code is laid out, how tools behave and how work is committed.
Read it before changing code. It applies to every agent, whatever its name says.

---

# Fork notes (konashevich/QymCAD — not for upstream PRs)

Everything below this line is operational context for this fork only. Cursor loads this
root `AGENTS.md` on every agent turn in this workspace. The feature branch
`feat/qymcad-drive` keeps the upstream-only `AGENTS.md` (the short paragraph above)
so a pull request to `QymIs-Tech/QymCAD` does not carry these notes.

Branch layout: [docs/fork/README.md](docs/fork/README.md).

## Why this fork exists

1. Upstream has no aarch64 Linux release, so this machine (Rockchip RK3588) builds and
   installs from source.
2. API / MCP work (`qymcad-drive` and related) is developed here with the intent to
   propose it upstream when it is ready.

## Daily work

Work on fork `main`. That branch has both the product code and these fork notes.
Open an upstream PR from `feat/qymcad-drive`, never from fork `main` or `local/ops`.

## ARM64 install notes (FriendlyELEC CM3588, Ubuntu 24.04)

There is no ready AppImage for aarch64. The `just pkg-linux` path also pulls
linuxdeploy / appimagetool for x86_64.

Ubuntu 24.04 apt ships OpenCASCADE 7.6. QymCAD `build.rs` needs the merged modules of
OCCT 7.8+ (TKDESTEP and neighbours), so the system package is not enough.

OCCT 7.9.3 (tag `V7_9_3`) was built from source with the same CMake flags as
`packaging/linux/Dockerfile`: shared libraries, Draw and Visualization off, FreeType,
Tk and OpenGL off at the kernel. Then:

```bash
cargo build --release --locked --bin qymcad
```

with `OCCT_INCLUDE_DIR` and `OCCT_LIB_DIR` pointed at that prefix. On eight RK3588
cores the kernel took about an hour; QymCAD release about a quarter of an hour. The
result was an aarch64 ELF around 51 MiB and the window opened.

RUNPATH on the binary alone is not enough: OCCT `.so` files load further OCCT `.so`
files without inheriting that RUNPATH. The launcher sets `LD_LIBRARY_PATH` to the
OCCT `lib` directory.

## Updating the installed app on this machine (no agent needed)

The running binary is built from a separate clean clone at
`/mnt/merged_ssd/app_data/qymcad/QymCAD` (upstream only). This AppDev tree is not used
for that install.

```bash
qymcad-update            # fetch upstream main, ff-only, rebuild if HEAD moved, install
qymcad-update --check    # fetch and report only
qymcad-update --force    # rebuild even when already at the tip
```

`qymcad-update` is `~/.local/bin/qymcad-update` ->
`/home/pi/os_admin/scripts/qymcad/update-from-upstream.sh`. Launch remains `qymcad`
(wrapper with `LD_LIBRARY_PATH` on the local OCCT lib).

## Personal agent scratch (not committed)

Untracked on purpose (repo `.gitignore` on `local/ops`, and the machine-wide
`*.local.md` ignore):

- `CLAUDE.local.md`
- `agent-api.local.md`
