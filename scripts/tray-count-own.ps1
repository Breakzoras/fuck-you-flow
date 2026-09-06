# Counts the hidden message windows a process owns for its notification icons.
# The tray-icon crate creates exactly one such window per tray icon, so this
# answers "how many tray icons does Lalia have?" without reading the shell.
#
# Usage: powershell -ExecutionPolicy Bypass -File scripts\tray-count-own.ps1 [processName]

param([string]$Name = "lalia")

$src = @'
using System;
using System.Text;
using System.Collections.Generic;
using System.Runtime.InteropServices;
public class WinEnum {
  delegate bool EnumProc(IntPtr h, IntPtr l);
  [DllImport("user32.dll")] static extern bool EnumWindows(EnumProc cb, IntPtr l);
  [DllImport("user32.dll")] static extern bool EnumChildWindows(IntPtr p, EnumProc cb, IntPtr l);
  [DllImport("user32.dll")] static extern uint GetWindowThreadProcessId(IntPtr h, out uint pid);
  [DllImport("user32.dll")] static extern int GetClassName(IntPtr h, StringBuilder s, int n);
  [DllImport("user32.dll")] static extern int GetWindowText(IntPtr h, StringBuilder s, int n);
  [DllImport("user32.dll")] static extern bool IsWindowVisible(IntPtr h);
  public static List<string> ForPid(uint target) {
    var list = new List<string>();
    EnumProc collect = (h, l) => {
      uint pid; GetWindowThreadProcessId(h, out pid);
      if (pid == target) {
        var c = new StringBuilder(256); GetClassName(h, c, 256);
        var t = new StringBuilder(256); GetWindowText(h, t, 256);
        list.Add("class='" + c + "' title='" + t + "' visible=" + IsWindowVisible(h));
      }
      return true;
    };
    EnumWindows(collect, IntPtr.Zero);
    EnumChildWindows(IntPtr.Zero, collect, IntPtr.Zero);
    return list;
  }
}
'@
Add-Type -TypeDefinition $src

$procs = Get-Process $Name -ErrorAction SilentlyContinue
if (-not $procs) { Write-Output "$Name is not running"; exit }
foreach ($p in $procs) {
  Write-Output "pid=$($p.Id)"
  $wins = [WinEnum]::ForPid([uint32]$p.Id)
  $wins | Sort-Object -Unique | ForEach-Object { Write-Output "  $_" }
  # EnumWindows and EnumChildWindows(NULL) both walk the top-level windows, so
  # every window is listed twice. Deduplicate before counting.
  $trayWins = @($wins | Sort-Object -Unique | Where-Object { $_ -match "tray_icon_app" })
  Write-Output "  -> tray-like windows: $($trayWins.Count)"
}
