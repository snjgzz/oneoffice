# gridcraft embed patch

`vendor/gridcraft` is a submodule of [storytold/gridcraft](https://github.com/storytold/gridcraft), pinned to `e6b6b349708ab4978cb3f061f6bd6ecbc25667cb` (main, 2026-10-15). OneOffice does not fork the crates into this workspace. `crates/shell` depends on `gridcraft-ui-egui` and `gridcraft-engine` by path.

`gridcraft-embed.patch` is the only local change. Apply it after checkout:

```sh
git submodule update --init --recursive
tools/apply-gridcraft-patch.sh
```

CI does the same. The submodule working tree is then dirty on purpose: the gitlink stays on the upstream commit, and the patch file is what we review and rebase.

## What the patch changes

`SheetApp.embed_salt: Option<u64>`

- `Some` skips the Quick Access title bar (window drag, centered document title) and does not send `ViewportCommand::Title`.
- `Some` installs fonts without `theme::apply`, and does not change the host's theme preference or zoom. Ribbon, formula bar, grid, sheet tabs and status bar still paint into the parent `Ui` (egui 0.36 panels already layout inside that `Ui`).
- Persistent panel, editor, dialog and popup ids go through `SheetApp::scope`, so two open spreadsheet tabs do not share egui widget memory.
- `theme::set_host_fallback_fonts` merges a host face into every font chain. OneOffice passes its Noto Sans SC subset. Role chains keep it at the end (Latin cell fonts still win). Proportional and monospace put it first, which is what the shell chrome uses.

Unpatched `SheetApp` (desktop gridcraft, `embed_salt == None`) keeps the previous behavior, including its own tests' font chains.

## Updating upstream

```sh
cd vendor/gridcraft
git fetch origin
git checkout <new-commit>
cd ../..
tools/apply-gridcraft-patch.sh   # fix conflicts, then:
git -C vendor/gridcraft diff > patches/gridcraft-embed.patch
```

Commit the new gitlink and the regenerated patch together. Do not commit the patched submodule tree as its own commit: that SHA is not on the upstream remote, so CI cannot fetch it.
