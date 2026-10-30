# Velmora remote config

The app downloads `registry.json` from this branch on every launch and
layers it over the UniverseAPI Engine's built-in defaults. A change here
reaches every installed copy on its next launch, with no app release.

Raw URL used by the app:

```
https://raw.githubusercontent.com/ashfaqizop/Velmora/config/registry.json
```

## Common edits

The file can be partial: any key you leave out keeps the engine's default.

| Situation | Edit |
|---|---|
| HDHub domain died and auto-resolution is wrong | set `hdhub.force_mirror` to the new mirror, e.g. `"https://new12.hdhub4u.xyz"`; set it back to `null` once auto-resolution recovers |
| Any other site moved | put the new base URL first in `mirrors.<provider>`; keep the old one after it as a fallback |
| A hoster renamed itself (`hubcloud` → `hubkloud`) | add `"hubkloud": "hubcloud"` to `host_aliases`. A TLD-only change (`hubcloud.ist` → `hubcloud.foo`) needs nothing |
| A provider is broken for everyone | add its id to `disabled`, e.g. `["coflix"]` |

Provider ids: `hd_hub`, `hi_anime`, `zoko_anime`, `movie_box`, `four_k_hd_hub`,
`dramachi`, `anime_sama`, `french_stream`, `coflix`, `yts`, `eztv`, `nyaa`.

Bump `version` only for a schema change the engine has to support first;
the engine refuses a version newer than it knows.

Never put secrets here. This branch is public; TMDB credentials reach the
app at build time from the repository secrets.
