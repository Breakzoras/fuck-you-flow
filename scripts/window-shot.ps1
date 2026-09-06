# Saves a screenshot of one top-level window to a PNG, so the dashboard can be
# checked from a script after a restart. Finds the window by exact title through
# EnumWindows and renders it with PrintWindow, so another window on top does not
# end up in the picture.
#
# Usage: powershell -ExecutionPolicy Bypass -File scripts\window-shot.ps1 -Title Lalia -Out C:\tmp\lalia.png

param(
  [string]$Title = "Fuck You Flow",
  [string]$Out = "$env:TEMP\window-shot.png"
)

$src = @'
using System;
using System.Text;
using System.Runtime.InteropServices;
public class WinShot {
  [StructLayout(LayoutKind.Sequential)] public struct RECT { public int L, T, R, B; }
  delegate bool EnumProc(IntPtr h, IntPtr l);
  [DllImport("user32.dll")] static extern bool EnumWindows(EnumProc cb, IntPtr l);
  [DllImport("user32.dll")] static extern int GetWindowText(IntPtr h, StringBuilder s, int n);
  [DllImport("user32.dll")] public static extern bool GetWindowRect(IntPtr h, out RECT r);
  [DllImport("user32.dll")] public static extern bool IsWindowVisible(IntPtr h);
  [DllImport("user32.dll")] public static extern bool PrintWindow(IntPtr h, IntPtr hdc, uint flags);
  public static IntPtr FindByTitle(string title) {
    IntPtr found = IntPtr.Zero;
    EnumProc cb = (h, l) => {
      var t = new StringBuilder(256); GetWindowText(h, t, 256);
      if (t.ToString() == title) { found = h; return false; }
      return true;
    };
    EnumWindows(cb, IntPtr.Zero);
    return found;
  }
}
'@
Add-Type -TypeDefinition $src
Add-Type -AssemblyName System.Drawing

$h = [WinShot]::FindByTitle($Title)
if ($h -eq [IntPtr]::Zero) { Write-Output "window '$Title' not found"; exit 2 }
if (-not [WinShot]::IsWindowVisible($h)) { Write-Output "window '$Title' is hidden"; exit 3 }
$r = New-Object WinShot+RECT
[void][WinShot]::GetWindowRect($h, [ref]$r)
$w = $r.R - $r.L; $hgt = $r.B - $r.T
if ($w -le 0 -or $hgt -le 0) { Write-Output "empty rect"; exit 4 }
$bmp = New-Object System.Drawing.Bitmap $w, $hgt
$g = [System.Drawing.Graphics]::FromImage($bmp)
$hdc = $g.GetHdc()
# 2 = PW_RENDERFULLCONTENT, needed for windows drawn by a browser engine (WebView2)
$ok = [WinShot]::PrintWindow($h, $hdc, 2)
$g.ReleaseHdc($hdc)
$bmp.Save($Out, [System.Drawing.Imaging.ImageFormat]::Png)
$g.Dispose(); $bmp.Dispose()
Write-Output "saved $Out ($w x $hgt, printwindow=$ok)"
