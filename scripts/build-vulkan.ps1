# Builds whisper.cpp with the Vulkan backend, which accelerates on AMD, Intel
# and NVIDIA cards alike. The official project ships only CPU and CUDA binaries
# for Windows, so this produces the missing piece ourselves from the same tag we
# already ship (b4938 / v1.9.3).
#
# Needs: Visual Studio 2022 Build Tools, the LunarG Vulkan SDK, CMake.
# Usage: powershell -ExecutionPolicy Bypass -File scripts\build-vulkan.ps1

$ErrorActionPreference = "Stop"
$src = "C:\Claude Projects\lalia\vendor\whisper.cpp-src"
$out = "C:\Claude Projects\lalia\vendor\whisper-vulkan"
$sdk = "C:\VulkanSDK\current"
$cmake = (Get-ChildItem "C:\Program Files*\Microsoft Visual Studio\2022\*\Common7\IDE\CommonExtensions\Microsoft\CMake\CMake\bin\cmake.exe" | Select-Object -First 1).FullName

$env:VULKAN_SDK = $sdk
$env:PATH = "$sdk\Bin;$env:PATH"

Write-Output "cmake: $cmake"
Write-Output "vulkan sdk: $sdk"

Set-Location $src
& $cmake -B build-vulkan -DGGML_VULKAN=ON -DBUILD_SHARED_LIBS=ON `
    -DWHISPER_BUILD_TESTS=OFF -DWHISPER_BUILD_EXAMPLES=ON -DWHISPER_BUILD_SERVER=ON `
    -DCMAKE_BUILD_TYPE=Release
if ($LASTEXITCODE -ne 0) { throw "configure failed" }

& $cmake --build build-vulkan --config Release --parallel
if ($LASTEXITCODE -ne 0) { throw "build failed" }

# collect what the app needs next to whisper-server.exe
New-Item -ItemType Directory -Force -Path $out | Out-Null
$want = @("whisper-server.exe", "whisper.dll", "ggml.dll", "ggml-base.dll", "ggml-vulkan.dll") +
        (Get-ChildItem "$src\build-vulkan\bin\Release" -Filter "ggml-cpu*.dll" -ErrorAction SilentlyContinue | Select-Object -ExpandProperty Name)
foreach ($f in ($want | Select-Object -Unique)) {
    $p = Join-Path "$src\build-vulkan\bin\Release" $f
    if (Test-Path $p) { Copy-Item $p $out -Force } else { Write-Output "missing: $f" }
}
Get-ChildItem $out | Select-Object Name, @{n='MB';e={[math]::Round($_.Length/1MB,2)}} | Format-Table -AutoSize
Write-Output ("total MB: " + [int](((Get-ChildItem $out | Measure-Object Length -Sum).Sum)/1MB))
