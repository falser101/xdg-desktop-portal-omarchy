# Upstream proposal materials (Path B)

Assets and drafts for offering this project to Omarchy as an **external package** (like `hyprland-preview-share-picker`), not as a monorepo dump into `basecamp/omarchy`.

## Files

| Path | Purpose |
|------|---------|
| [discussion-draft.md](discussion-draft.md) | English text for a Suggestions discussion |
| [packaging-brief.md](packaging-brief.md) | Maintainer-oriented packaging / wiring notes |
| [assets/](assets/) | Dialog screenshots for the discussion |
| `../../scripts/capture-upstream-assets.sh` | Re-capture stills on a live Omarchy session |

## Screenshots checklist

| File | Status | Notes |
|------|--------|-------|
| `open.png` | done | FileChooser Open |
| `save.png` | done | FileChooser Save |
| `screenshot.png` | done | Screenshot portal |
| `access.png` | done | Access dialog |
| `app-chooser.png` | done | AppChooser |
| `share-picker.png` | done | Omarchy ScreenCast picker |
| `*-fullscreen.png` | done | Same moments, full monitor |

## How to re-capture

```bash
./scripts/capture-upstream-assets.sh
```

Requires an active Hyprland/Omarchy session with `xdg-desktop-portal-omarchy` running and `wtype` / `grim` available.

## Suggested next steps

1. Review stills (crop / redo any empty preview if needed).
2. Push `docs/upstream/` to GitHub.
3. Open a Suggestions discussion; attach PNGs from `assets/` (GitHub discussion upload — `gh` cannot attach images).
4. Link [#7944](https://github.com/basecamp/omarchy/issues/7944) and the packaging brief.
5. Wait for maintainer interest before any `basecamp/omarchy` wiring PR.
