# Lists every icon in the Windows notification area with the process that owns
# it, so a duplicate can be told apart from a "ghost" left by a killed process.
# Reads the shell toolbar's button data out of explorer.exe (read-only).
#
# Usage: powershell -ExecutionPolicy Bypass -File scripts\tray-inspect.ps1

$src = @'
using System;
using System.Text;
using System.Runtime.InteropServices;
public class TrayInspect {
  [DllImport("user32.dll")] static extern IntPtr FindWindow(string c, string w);
  [DllImport("user32.dll")] static extern IntPtr FindWindowEx(IntPtr p, IntPtr c, string cls, string win);
  [DllImport("user32.dll")] static extern IntPtr SendMessage(IntPtr h, uint m, IntPtr wp, IntPtr lp);
  [DllImport("user32.dll")] static extern uint GetWindowThreadProcessId(IntPtr h, out uint pid);
  [DllImport("user32.dll")] static extern bool IsWindow(IntPtr h);
  [DllImport("user32.dll")] static extern int GetWindowText(IntPtr h, StringBuilder s, int n);
  [DllImport("kernel32.dll")] static extern IntPtr OpenProcess(uint a, bool i, uint pid);
  [DllImport("kernel32.dll")] static extern IntPtr VirtualAllocEx(IntPtr p, IntPtr a, IntPtr s, uint t, uint pr);
  [DllImport("kernel32.dll")] static extern bool VirtualFreeEx(IntPtr p, IntPtr a, IntPtr s, uint t);
  [DllImport("kernel32.dll")] static extern bool ReadProcessMemory(IntPtr p, IntPtr a, byte[] b, IntPtr s, out IntPtr r);
  [DllImport("kernel32.dll")] static extern bool CloseHandle(IntPtr h);
  const uint TB_BUTTONCOUNT = 0x0418;
  const uint TB_GETBUTTON = 0x0417;
  static IntPtr Bar(bool overflow) {
    if (overflow) {
      IntPtr o = FindWindow("NotifyIconOverflowWindow", null);
      return FindWindowEx(o, IntPtr.Zero, "ToolbarWindow32", null);
    }
    IntPtr t = FindWindow("Shell_TrayWnd", null);
    IntPtr n = FindWindowEx(t, IntPtr.Zero, "TrayNotifyWnd", null);
    IntPtr p = FindWindowEx(n, IntPtr.Zero, "SysPager", null);
    return FindWindowEx(p, IntPtr.Zero, "ToolbarWindow32", null);
  }
  public static string Dump(bool overflow) {
    IntPtr bar = Bar(overflow);
    if (bar == IntPtr.Zero) return (overflow ? "OVERFLOW" : "VISIBLE ") + " (toolbar not found)";
    uint pid; GetWindowThreadProcessId(bar, out pid);
    IntPtr hp = OpenProcess(0x0438, false, pid);
    if (hp == IntPtr.Zero) return "(cannot open shell process)";
    IntPtr rem = VirtualAllocEx(hp, IntPtr.Zero, (IntPtr)1024, 0x1000, 4);
    int count = (int)SendMessage(bar, TB_BUTTONCOUNT, IntPtr.Zero, IntPtr.Zero);
    var sb = new StringBuilder();
    sb.AppendLine((overflow ? "OVERFLOW" : "VISIBLE ") + " buttons=" + count);
    for (int i = 0; i < count; i++) {
      SendMessage(bar, TB_GETBUTTON, (IntPtr)i, rem);
      byte[] btn = new byte[32]; IntPtr rd;
      if (!ReadProcessMemory(hp, rem, btn, (IntPtr)32, out rd)) continue;
      long dwData = BitConverter.ToInt64(btn, 16);
      byte fsState = btn[8];
      if (dwData == 0) continue;
      byte[] td = new byte[32];
      if (!ReadProcessMemory(hp, (IntPtr)dwData, td, (IntPtr)32, out rd)) continue;
      IntPtr owner = (IntPtr)BitConverter.ToInt64(td, 0);
      uint uid = BitConverter.ToUInt32(td, 8);
      uint opid = 0; GetWindowThreadProcessId(owner, out opid);
      bool alive = IsWindow(owner);
      string name;
      try { name = System.Diagnostics.Process.GetProcessById((int)opid).ProcessName; } catch { name = "(dead)"; }
      var t = new StringBuilder(200); GetWindowText(owner, t, 200);
      bool hidden = (fsState & 0x08) != 0;
      sb.AppendLine("  #" + i + " pid=" + opid + " proc=" + name + " uid=" + uid + " aliveWindow=" + alive + " hiddenButton=" + hidden + " title='" + t.ToString() + "'");
    }
    VirtualFreeEx(hp, rem, IntPtr.Zero, 0x8000);
    CloseHandle(hp);
    return sb.ToString();
  }
}
'@
Add-Type -TypeDefinition $src
Write-Output ([TrayInspect]::Dump($false))
Write-Output ([TrayInspect]::Dump($true))
