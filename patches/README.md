# Embed patches

`vendor/gridcraft`, `vendor/wordcraft` and `vendor/deckcraft` are submodules. OneOffice does not add their crates to this workspace. `crates/shell` depends on each `*-ui-egui` and `*-engine` by path. The gitlink stays on the upstream commit; the patch file is what we review.

| submodule | pin | patch |
| --- | --- | --- |
| gridcraft | `e6b6b349708ab4978cb3f061f6bd6ecbc25667cb` | `gridcraft-embed.patch` |
| wordcraft | `e127a080a162eb40d7cc208c73421573399e8f3b` | `wordcraft-embed.patch` |
| deckcraft | `9859da39255c0262cd592bb57101d7f2fa961f72` | `deckcraft-embed.patch` |

Apply them after checkout:

```sh
git submodule update --init --recursive
tools/apply-gridcraft-patch.sh
tools/apply-wordcraft-patch.sh
tools/apply-deckcraft-patch.sh
```

CI does the same. Each working tree is then dirty on purpose.

## What the patches change

`embed_salt: Option<u64>` on `SheetApp`, `WordApp` and `SlideApp`.

- `Some` skips that app's title bar and does not send a viewport title (gridcraft and wordcraft; deckcraft has no viewport title).
- `Some` does not call the app's `theme::apply` or `set_zoom_factor`, and does not take over quit or zoom shortcuts. Ribbon, canvas and status bar still paint into the parent `Ui`.
- The shell also wraps each tab in `ui.push_id`, so panels and default widgets do not share egui memory. Dialog windows use `scope`, because those ids are built from `Id::new` and ignore the parent `Ui`.
- `theme::set_host_fallback_fonts` merges a host face into every font chain. OneOffice passes its Noto Sans SC subset. Role chains keep it at the end. Proportional and monospace put it first, which is what the shell chrome uses. Without this, each app's `set_fonts` would drop Noto.

File › Close while embedded sets `embed_closed` instead of leaving a blank document (gridcraft) or quitting the process (wordcraft). DeckCraft drops the presentation and the shell closes that tab. Unpatched apps (`embed_salt == None`) keep their previous behavior.

The three apps share one egui font map. Whichever tab last called `set_fonts` owns the named families. DeckCraft paints with `FontFamily::Name("bold")`, and gridcraft uses its own role families. Switching back to a tab whose families were replaced panics in epaint. On that switch the shell calls `prepare_embed_fonts` after `logic`. Each app reinstalls its fonts and skips painting for the pass egui needs before the new definitions are active.

## krilla

`deckcraft-pdf` depends on `craft-pdf`, which refuses crates.io `krilla` unless `CRAFT_UNWIND_POISONING` is present. `vendor/deckcraft/Cargo.toml` already patches `krilla` to craft-libs `c3487320b2d1de657b860b3c8abb44e1f2b32bd8`, but a `[patch]` in an excluded workspace does not apply to OneOffice. The same pin is repeated in the root `Cargo.toml`. WordCraft's PDF crate then links that fork too. The fork keeps the 0.8.2 API and adds `KrillaError::Poisoned`.

## Updating upstream

```sh
cd vendor/<name>
git fetch origin
git checkout <new-commit>
cd ../..
tools/apply-<name>-patch.sh   # fix conflicts, then:
git -C vendor/<name> diff > patches/<name>-embed.patch
```

Commit the new gitlink and the regenerated patch together. Do not commit the patched submodule tree as its own commit: that SHA is not on the upstream remote, so CI cannot fetch it.
