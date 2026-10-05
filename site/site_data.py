"""Release facts shared by every generated public page.

Installer size measured from the signed 0.9.14 setup.
Update these facts together when publishing a release.
"""
BASE = "https://fuckyouflow.app"
REPO = "https://github.com/Breakzoras/fuck-you-flow"
VERSION = "0.9.14"
SIZE_BYTES = 1730231254
SIZE = "1.7 GB"
SIZE_EL = "1,7 GB"
RELEASE_DATE = "2026-10-05"
CONTENT_DATE = "2026-09-26"
DL = f"{REPO}/releases/download/v{VERSION}/Fuck.You.Flow.Setup.{VERSION}.exe"
# The link preview every page shares, 1200 x 630. A new picture gets a new file name,
# so the networks that cache previews fetch it again. The 27 September 2026 card is
# the ad's first frame beside the app window; its source is og-card.html in the
# fyf-ad-2026-09 folder.
OG_IMAGE = f"{BASE}/assets/og-2026-09-27.png"
OG_ALT = ("The words $15 a month for Wispr Flow? in white and coral, with FU Flow: free voice "
          "typing underneath, beside the FU Flow app window on its ready screen.")
# Linux, full packages with the speech models inside. The small ones the
# updater fetches sit on the same release under other names.
# Linux can trail Windows: 0.9.13 shipped for Windows only, so the Ubuntu
# buttons keep pointing at the last Linux release.
LINUX_VERSION = "0.9.12"
LINUX_DEB = f"{REPO}/releases/download/v{LINUX_VERSION}/Fuck.You.Flow.Setup.{LINUX_VERSION}_amd64.deb"
LINUX_APPIMAGE = f"{REPO}/releases/download/v{LINUX_VERSION}/Fuck.You.Flow.Setup.{LINUX_VERSION}_amd64.AppImage"


def release_tokens(text, lang="en"):
    for key, value in {
        "VERSION": VERSION, "SIZE": SIZE_EL if lang == "el" else SIZE,
        "DOWNLOAD": DL, "RELEASE": f"{REPO}/releases/tag/v{VERSION}",
        "LINUX_DEB": LINUX_DEB, "LINUX_APPIMAGE": LINUX_APPIMAGE,
    }.items():
        text = text.replace("{{" + key + "}}", value)
    return text
