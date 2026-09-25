"""Release facts shared by every generated public page.

Installer size measured from the signed 0.9.11 setup.
Update these facts together when publishing a release.
"""
BASE = "https://fuckyouflow.app"
REPO = "https://github.com/Breakzoras/fuck-you-flow"
VERSION = "0.9.11"
SIZE_BYTES = 1729716694
SIZE = "1.7 GB"
SIZE_EL = "1,7 GB"
RELEASE_DATE = "2026-09-25"
CONTENT_DATE = "2026-09-25"
DL = f"{REPO}/releases/download/v{VERSION}/Fuck.You.Flow.Setup.{VERSION}.exe"


def release_tokens(text, lang="en"):
    for key, value in {
        "VERSION": VERSION, "SIZE": SIZE_EL if lang == "el" else SIZE,
        "DOWNLOAD": DL, "RELEASE": f"{REPO}/releases/tag/v{VERSION}",
    }.items():
        text = text.replace("{{" + key + "}}", value)
    return text
