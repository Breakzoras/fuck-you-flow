# Cutting a release, and how an installed copy updates itself

Two different files leave this project, and they are not interchangeable.

| File | Who downloads it | What is inside | Size |
|---|---|---|---|
| `Fuck.You.Flow.Setup.<version>.exe` | a new user, from fuckyouflow.app | program, engine, models | about 1.7 GB |
| `Fuck.You.Flow.Update.<version>.exe` | an installed copy, on its own | program and engine | about 68 MB |

The update is small because the models do not live in the install folder any
more. The first time the app runs after a full install it moves them to
`%LOCALAPPDATA%\Lalia\models` (`models::migrate_bundled_models`). That move is a
rename on the same disk, so it costs nothing whatever the size. It has to happen
because the Windows installer removes the previous version before writing the
new one, and anything left in the install folder goes with it.

If the move fails, which only happens when the user installed onto another disk
and the copy could not be made, the app knows: `models_are_external()` returns
false, the update bar says the download is the big one, and the release for that
case has to be the full installer.

## The signing key

`~/.tauri/fuckyouflow.key`, generated on 7 September 2026. It is not in the
repository and it is not in any backup that leaves this machine. **Losing it
means no installed copy can ever be updated again**, because the app refuses an
installer that is not signed by the key whose public half is baked into
`src-tauri/tauri.conf.json`.

## Steps

```bash
# 1. version number in both files, same value
#    src-tauri/tauri.conf.json  ->  "version"
#    src-tauri/Cargo.toml       ->  version

# 2. the small one, for people who already have the app
#    The bundler reads TAURI_SIGNING_PRIVATE_KEY and nothing else. Measured on
#    8 September 2026: with only TAURI_SIGNING_PRIVATE_KEY_PATH set, the build
#    finishes and then says "a public key has been found, but no private key",
#    and no .sig comes out.
export TAURI_SIGNING_PRIVATE_KEY="$(cat ~/.tauri/fuckyouflow.key)"
export TAURI_SIGNING_PRIVATE_KEY_PASSWORD=
python scripts/stage-bundle.py --slim
pnpm tauri build
python scripts/make-update-manifest.py --notes "what changed, one line"

# If a build came out unsigned, the file can be signed on its own:
#   pnpm tauri signer sign -f ~/.tauri/fuckyouflow.key -p "" <the setup exe>

# 3. the big one, for the site
python scripts/stage-bundle.py
pnpm tauri build

# 4. upload both to the release, then publish the site.
#    make-update-manifest.py prints the exact commands.
```

The manifest goes up **last**. An app that reads `latest.json` before the
installer is on GitHub gets a broken download.

## Where the pieces live

- The message: `https://fuckyouflow.app/updates/latest.json`, built by
  `scripts/make-update-manifest.py` into `site/updates/`, published by
  `site/deploy.sh`.
- The files: GitHub releases on `Breakzoras/fuck-you-flow`.
- The check: fifteen seconds after the app starts, and by hand from
  Settings, General, Updates. Nothing downloads until the user presses the
  button on the bar.

## Compression

`bundle.windows.nsis.compression` is `none`. The payload is 1.6 GB of model
weights that are already compressed, and LZMA squeezed the installer from
1.73 GB down to 1.46 GB, a saving of 16 percent paid for by minutes of
unpacking on every single install.
