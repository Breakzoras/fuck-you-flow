"""Release facts shared by every generated public page.

Installer size measured from the signed 0.9.12 setup.
Update these facts together when publishing a release.
"""
BASE = "https://fuckyouflow.app"
REPO = "https://github.com/Breakzoras/fuck-you-flow"
VERSION = "0.9.12"
SIZE_BYTES = 1729796054
SIZE = "1.7 GB"
SIZE_EL = "1,7 GB"
RELEASE_DATE = "2026-09-26"
CONTENT_DATE = "2026-09-26"
DL = f"{REPO}/releases/download/v{VERSION}/Fuck.You.Flow.Setup.{VERSION}.exe"
# Linux, full packages with the speech models inside. The small ones the
# updater fetches sit on the same release under other names.
LINUX_DEB = f"{REPO}/releases/download/v{VERSION}/Fuck.You.Flow.Setup.{VERSION}_amd64.deb"
LINUX_APPIMAGE = f"{REPO}/releases/download/v{VERSION}/Fuck.You.Flow.Setup.{VERSION}_amd64.AppImage"


def release_tokens(text, lang="en"):
    for key, value in {
        "VERSION": VERSION, "SIZE": SIZE_EL if lang == "el" else SIZE,
        "DOWNLOAD": DL, "RELEASE": f"{REPO}/releases/tag/v{VERSION}",
        "LINUX_DEB": LINUX_DEB, "LINUX_APPIMAGE": LINUX_APPIMAGE,
    }.items():
        text = text.replace("{{" + key + "}}", value)
    return text
