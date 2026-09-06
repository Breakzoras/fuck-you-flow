# Clears "ghost" tray icons left behind when a process was killed without
# removing its notification icon. Windows only drops such an icon when the mouse
# passes over it, so this sends synthetic mouse-move messages across both the
# visible notification area and the overflow flyout.
#
# Usage: powershell -ExecutionPolicy Bypass -File scripts\tray-sweep.ps1

$src = @'
using System;
using System.Runtime.InteropServices;
public class TraySweep {
  [DllImport("user32.dll")] public static extern IntPtr FindWindow(string c, string w);
  [DllImport("user32.dll")] public static extern IntPtr FindWindowEx(IntPtr p, IntPtr c, string cls, string win);
  [DllImport("user32.dll")] public static extern IntPtr SendMessage(IntPtr h, uint m, IntPtr wp, IntPtr lp);
  [DllImport("user32.dll")] public static extern bool GetClientRect(IntPtr h, out RECT r);
  [StructLayout(LayoutKind.Sequential)] public struct RECT { public int L, T, R, B; }
  const uint WM_MOUSEMOVE = 0x0200;
  public static int Sweep(IntPtr bar) {
    if (bar == IntPtr.Zero) return 0;
    RECT r; if (!GetClientRect(bar, out r)) return 0;
    int n = 0;
    for (int x = 0; x < r.R; x += 8) for (int y = 0; y < r.B; y += 8) {
      SendMessage(bar, WM_MOUSEMOVE, IntPtr.Zero, (IntPtr)((y << 16) | x)); n++;
    }
    return n;
  }
  public static IntPtr Visible() {
    IntPtr t = FindWindow("Shell_TrayWnd", null);
    IntPtr n = FindWindowEx(t, IntPtr.Zero, "TrayNotifyWnd", null);
    IntPtr p = FindWindowEx(n, IntPtr.Zero, "SysPager", null);
    return FindWindowEx(p, IntPtr.Zero, "ToolbarWindow32", null);
  }
  public static IntPtr Overflow() {
    IntPtr o = FindWindow("NotifyIconOverflowWindow", null);
    return FindWindowEx(o, IntPtr.Zero, "ToolbarWindow32", null);
  }
}
'@
Add-Type -TypeDefinition $src
$v = [TraySweep]::Sweep([TraySweep]::Visible())
$o = [TraySweep]::Sweep([TraySweep]::Overflow())
Write-Output "tray swept: $v visible, $o overflow"
