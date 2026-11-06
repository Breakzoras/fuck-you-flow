# Builds the Windows engine of FU Flow 0.9.20 and later: whisper.cpp v1.9.5
# with every backend as a separate DLL that whisper-server loads at runtime.
#
# Why (report of 6 November 2026, 0.9.19): the old build linked ggml.dll to
# ggml-vulkan.dll, which needs vulkan-1.dll, so a PC without a Vulkan driver got
# 0xc0000135 and could not dictate at all. Its CPU part was also compiled for
# the build PC (GGML_NATIVE, AVX2), so a processor without AVX2 would crash next.
# Here ggml-vulkan.dll is tried and skipped when Vulkan is missing, and the best
# of ten ggml-cpu-<level>.dll is picked for the processor (ggml-cpu-x64 runs on
# any 64-bit one). The Visual C++ runtime is copied next to it, because a clean
# Windows does not have it and the app itself does not need it.
#
# Low priority, at most 4 compiler processes. Writes only build-dl and the
# output folder; the 0.9.15-0.9.19 engine (vendor\whisper-vulkan-1.9.5) stays
# as it is, as the way back.
#
# Usage: powershell -ExecutionPolicy Bypass -File scripts\build-engine-dl.ps1
$ErrorActionPreference = 'Stop'
(Get-Process -Id $PID).PriorityClass = 'Idle'
$src = 'C:\Claude Projects\lalia\vendor\whisper.cpp-src-1.9.5'
$bld = Join-Path $src 'build-dl'
$out = 'C:\Claude Projects\lalia\vendor\whisper-dl-1.9.5'
$vs = 'C:\Program Files (x86)\Microsoft Visual Studio\2022\BuildTools'
$cmake = "$vs\Common7\IDE\CommonExtensions\Microsoft\CMake\CMake\bin\cmake.exe"
$env:VULKAN_SDK = 'C:\VulkanSDK\current'
$env:PATH = "$env:VULKAN_SDK\Bin;$env:PATH"
$env:CL = '/MP4'

& $cmake -S $src -B $bld -G 'Visual Studio 17 2022' -A x64 `
  -DBUILD_SHARED_LIBS=ON -DGGML_BACKEND_DL=ON -DGGML_CPU_ALL_VARIANTS=ON `
  -DGGML_NATIVE=OFF -DGGML_VULKAN=ON -DGGML_CCACHE=OFF `
  -DWHISPER_BUILD_SERVER=ON -DWHISPER_BUILD_EXAMPLES=ON -DWHISPER_BUILD_TESTS=OFF
if ($LASTEXITCODE -ne 0) { throw 'configure failed' }
& $cmake --build $bld --config Release -- /m:1 /nologo /v:minimal
if ($LASTEXITCODE -ne 0) { throw 'build failed' }

New-Item -ItemType Directory -Force -Path $out | Out-Null
$rel = Join-Path $bld 'bin\Release'
$want = @('whisper-server.exe', 'whisper.dll', 'ggml.dll', 'ggml-base.dll', 'ggml-vulkan.dll') +
        (Get-ChildItem $rel -Filter 'ggml-cpu-*.dll' | Select-Object -ExpandProperty Name)
foreach ($f in $want) { Copy-Item (Join-Path $rel $f) $out -Force }

# The Visual C++ runtime of the same toolset, app-local (Microsoft allows this).
$redist = Get-ChildItem "$vs\VC\Redist\MSVC" -Directory | Where-Object { $_.Name -match '^\d' } |
          Sort-Object { [version]$_.Name } | Select-Object -Last 1
foreach ($f in 'msvcp140.dll', 'vcruntime140.dll', 'vcruntime140_1.dll') {
  Copy-Item (Join-Path $redist.FullName "x64\Microsoft.VC143.CRT\$f") $out -Force
}
Copy-Item (Join-Path $redist.FullName 'x64\Microsoft.VC143.OpenMP\vcomp140.dll') $out -Force
"runtime from $($redist.Name)"
Get-ChildItem $out | Select-Object Name, @{n = 'MB'; e = { [math]::Round($_.Length / 1MB, 2) } } | Format-Table -AutoSize
'total MB: ' + [int](((Get-ChildItem $out | Measure-Object Length -Sum).Sum) / 1MB)
