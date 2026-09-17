"""Writes site/updates/latest.json, the file the installed app reads to learn
that a new version exists.

The app asks https://fuckyouflow.app/updates/latest.json, and that answer points
at the installer on GitHub. So the message lives on our own domain and the
gigabytes live on GitHub, which is free and fast.

The installer must be signed, or the app refuses it. Signing happens during the
build, when TAURI_SIGNING_PRIVATE_KEY_PATH points at the private key:

    export TAURI_SIGNING_PRIVATE_KEY_PATH=~/.tauri/fuckyouflow.key
    export TAURI_SIGNING_PRIVATE_KEY_PASSWORD=
    python scripts/stage-bundle.py --slim
    pnpm tauri build
    python scripts/make-update-manifest.py --notes "what changed"

The private key never goes into the repository. Losing it means no installed
copy can ever be updated again.

Run from the project root.
"""
import argparse
import datetime
import glob
import io
import json
import os
import re

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
REPO = "Breakzoras/fuck-you-flow"


def names_version(path, version):
    """Whether the file name carries exactly this version: 0.9.8 matches
    "Fuck You Flow_0.9.8_x64-setup.exe" but not 0.9.80 or 10.9.8."""
    pattern = r"(?<!\d)(?<!\d\.)" + re.escape(version) + r"(?!\d)(?!\.\d)"
    return re.search(pattern, os.path.basename(path)) is not None


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--notes", default="", help="what changed, shown to the user")
    ap.add_argument("--tag", default="", help="the GitHub release tag, default v<version>")
    ap.add_argument("--asset", default="", help="the file name on the release, default Fuck.You.Flow.Update.<version>.exe")
    ap.add_argument("--exe", default="", help="the installer to point at; its .sig must sit beside it. "
                                              "Default: the signed one for the version in tauri.conf.json. Give this when "
                                              "the slim build has been moved aside and the full one has taken its place. "
                                              "Its file name must still carry that version.")
    args = ap.parse_args()

    conf = json.load(io.open(os.path.join(ROOT, "src-tauri", "tauri.conf.json"), encoding="utf-8"))
    version = conf["version"]
    tag = args.tag or "v" + version

    # Tauri names the installer "<product>_<version>_x64-setup.exe" and never
    # empties the bundle folder, so signed installers of earlier versions stay
    # beside the new one. Choosing the newest file meant that a build which
    # failed, or ran without the signing key, left the previous version's
    # installer to be published under this version's number: every installed
    # copy would then "update" to the same program and be offered it again
    # forever. Only an installer whose file name carries this version is used.
    if args.exe:
        exe_path = os.path.abspath(args.exe)
        sig_path = exe_path + ".sig"
        if not names_version(exe_path, version):
            raise SystemExit(
                "%s does not carry version %s in its name.\n"
                "tauri.conf.json says %s. Build that version, or point --exe at its installer."
                % (exe_path, version, version)
            )
        if not os.path.exists(sig_path):
            raise SystemExit("no signature beside %s" % exe_path)
    else:
        nsis = os.path.join(ROOT, "src-tauri", "target", "release", "bundle", "nsis")
        sigs = glob.glob(os.path.join(nsis, "*-setup.exe.sig"))
        if not sigs:
            raise SystemExit(
                "no signature in %s\n"
                "The build did not sign the installer. Set TAURI_SIGNING_PRIVATE_KEY and build again." % nsis
            )
        mine = sorted(p for p in sigs if names_version(p, version))
        if not mine:
            raise SystemExit(
                "no signed installer for version %s in %s\n"
                "Signed installers found: %s\n"
                "The %s build did not finish or was not signed. Build it again; "
                "an older installer must never go out under this version."
                % (version, nsis, ", ".join(sorted(os.path.basename(p) for p in sigs)), version)
            )
        if len(mine) > 1:
            raise SystemExit(
                "more than one signed installer for version %s in %s:\n  %s\n"
                "Say which one with --exe." % (version, nsis, "\n  ".join(os.path.basename(p) for p in mine))
            )
        sig_path = mine[0]
        exe_path = sig_path[: -len(".sig")]
    if not os.path.exists(exe_path):
        raise SystemExit("signature without an installer: %s" % sig_path)

    size = os.path.getsize(exe_path)
    signature = io.open(sig_path, encoding="utf-8").read().strip()
    # The release keeps its own naming (Fuck.You.Flow.Setup.0.9.1.exe), so the
    # name on GitHub is decided here and the upload below uses the same one.
    asset = args.asset or "Fuck.You.Flow.Update.%s.exe" % version
    url = "https://github.com/%s/releases/download/%s/%s" % (REPO, tag, asset)

    manifest = {
        "version": version,
        "notes": args.notes,
        "pub_date": datetime.datetime.now(datetime.timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ"),
        "platforms": {
            "windows-x86_64": {"signature": signature, "url": url},
        },
    }
    out_dir = os.path.join(ROOT, "site", "updates")
    os.makedirs(out_dir, exist_ok=True)
    out = os.path.join(out_dir, "latest.json")
    io.open(out, "w", encoding="utf-8").write(json.dumps(manifest, indent=2, ensure_ascii=False) + "\n")

    print("version   %s" % version)
    print("installer %s (%.0f MB)" % (os.path.basename(exe_path), size / 1048576.0))
    print("url       %s" % url)
    print("written   %s" % out)
    print()
    print("Then, in this order:")
    print('  gh release upload %s "%s#%s" --clobber --repo %s' % (tag, exe_path, asset, REPO))
    print("  bash site/deploy.sh")
    print("The manifest must go up last, or an app could read it before the file exists.")


if __name__ == "__main__":
    main()
