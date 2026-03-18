[CmdletBinding()]
Param(
    [Parameter()][Alias('i')][switch]$Install,
    [Parameter()][Alias('h')][switch]$Help,
    [Parameter()][Alias('a')][string]$Architecture,
    [Parameter()][string]$Name
)

. "$PSScriptRoot/lib/workspace.ps1"

# https://stackoverflow.com/questions/57949031/powershell-script-stops-if-program-fails-like-bash-set-o-errexit
$ErrorActionPreference = 'Stop'
$PSNativeCommandUseErrorActionPreference = $true

$buildSuccess = $false

$OSArchitecture = switch ([System.Runtime.InteropServices.RuntimeInformation]::OSArchitecture) {
    "X64" { "x86_64" }
    "Arm64" { "aarch64" }
    default { throw "Unsupported architecture" }
}

$Architecture = if ($Architecture) {
    $Architecture
} else {
    $OSArchitecture
}

$CargoOutDir = "./target/$Architecture-pc-windows-msvc/release"

function Get-VSArch {
    param(
        [string]$Arch
    )

    switch ($Arch) {
        "x86_64" { "amd64" }
        "aarch64" { "arm64" }
    }
}

$vsPath = & "${env:ProgramFiles(x86)}\Microsoft Visual Studio\Installer\vswhere.exe" -latest -property installationPath
if (-not $vsPath) {
    # Fallback to a common default if vswhere is not found or fails
    $vsPath = "C:\Program Files\Microsoft Visual Studio\2022\Community"
}
$vsDevShell = Join-Path $vsPath "Common7\Tools\Launch-VsDevShell.ps1"

if (Test-Path $vsDevShell) {
    Write-Host "🚀 Launching VS Developer Shell from $vsDevShell"
    Push-Location
    & $vsDevShell -Arch (Get-VSArch -Arch $Architecture) -HostArch (Get-VSArch -Arch $OSArchitecture) -SkipAutomaticLocation
    Pop-Location
} else {
    Write-Warning "Could not find Launch-VsDevShell.ps1 at $vsDevShell. Build may fail if environment is not set."
}

$target = "$Architecture-pc-windows-msvc"

if ($Help) {
    Write-Output "Usage: test.ps1 [-Install] [-Help]"
    Write-Output "Build the installer for Windows.\n"
    Write-Output "Options:"
    Write-Output "  -Architecture, -a Which architecture to build (x86_64 or aarch64)"
    Write-Output "  -Install, -i      Run the installer after building."
    Write-Output "  -Help, -h         Show this help message."
    exit 0
}

Push-Location -Path crates/zed-custom
$channel = Get-Content "RELEASE_CHANNEL"
$env:ZED_RELEASE_CHANNEL = $channel
$env:RELEASE_CHANNEL = $channel
Pop-Location

function CheckEnvironmentVariables {
    if(-not $env:CI) {
        return
    }

    $buildVars = @('ZED_WORKSPACE', 'RELEASE_VERSION', 'ZED_RELEASE_CHANNEL')
    foreach ($var in $buildVars) {
        if (-not (Test-Path "env:$var")) {
            Write-Error "$var is not set"
            exit 1
        }
    }

    $signingVars = @(
        'AZURE_TENANT_ID', 'AZURE_CLIENT_ID', 'AZURE_CLIENT_SECRET',
        'ACCOUNT_NAME', 'CERT_PROFILE_NAME', 'ENDPOINT',
        'FILE_DIGEST', 'TIMESTAMP_DIGEST', 'TIMESTAMP_SERVER'
    )

    $allSigningVarsSet = $true
    foreach ($var in $signingVars) {
        if (-not (Test-Path "env:$var")) {
            $allSigningVarsSet = $false
            break
        }
    }

    $env:ZED_SIGNING_READY = if ($allSigningVarsSet) { "true" } else { "false" }
    if ($allSigningVarsSet -eq $false) {
        Write-Warning "Signing secrets are missing. The build will proceed but binaries will be unsigned."
    }
}

function PrepareForBundle {
    if (Test-Path "$innoDir") {
        Remove-Item -Path "$innoDir" -Recurse -Force
    }
    New-Item -Path "$innoDir" -ItemType Directory -Force
    Copy-Item -Path "$env:ZED_WORKSPACE\crates\zed-custom\resources\windows\*" -Destination "$innoDir" -Recurse -Force
    New-Item -Path "$innoDir\make_appx" -ItemType Directory -Force
    New-Item -Path "$innoDir\appx" -ItemType Directory -Force
    New-Item -Path "$innoDir\bin" -ItemType Directory -Force
    New-Item -Path "$innoDir\tools" -ItemType Directory -Force

    rustup target add $target
}

function GenerateLicenses {
    $oldErrorActionPreference = $ErrorActionPreference
    $ErrorActionPreference = 'Continue'
    . $PSScriptRoot/generate-licenses.ps1
    $ErrorActionPreference = $oldErrorActionPreference
}

function BuildZedAndItsFriends {
    Write-Output "Building zed-custom and its friends, for channel: $channel"
    # Build zed-custom.exe, cli.exe and auto_update_helper.exe
    cargo build --release --package zed-custom --package cli --package auto_update_helper --target $target
    Copy-Item -Path ".\$CargoOutDir\zed-custom.exe" -Destination "$innoDir\zed-custom.exe" -Force
    Copy-Item -Path ".\$CargoOutDir\cli.exe" -Destination "$innoDir\cli.exe" -Force
    Copy-Item -Path ".\$CargoOutDir\auto_update_helper.exe" -Destination "$innoDir\auto_update_helper.exe" -Force
    # Build explorer_command_injector.dll
    switch ($channel) {
        "stable" {
            cargo build --release --features stable --no-default-features --package explorer_command_injector --target $target
        }
        "preview" {
            cargo build --release --features preview --no-default-features --package explorer_command_injector --target $target
        }
        default {
            cargo build --release --package explorer_command_injector --target $target
        }
    }
    Copy-Item -Path ".\$CargoOutDir\explorer_command_injector.dll" -Destination "$innoDir\zed_explorer_command_injector.dll" -Force
}

function BuildRemoteServer {
    Write-Output "Building remote_server for $target"
    cargo build --release --package remote_server --target $target

    # Create zipped remote server binary
    $remoteServerSrc = (Resolve-Path ".\$CargoOutDir\remote_server.exe").Path

    if ($env:CI -and $env:ZED_SIGNING_READY -eq "true") {
        Write-Output "Code signing remote_server.exe"
        & "$innoDir\sign.ps1" $remoteServerSrc
    }

    $remoteServerDst = "$env:ZED_WORKSPACE\target\zed-custom-remote-server-windows-$Architecture.zip"
    Write-Output "Compressing remote_server to $remoteServerDst"
    Compress-Archive -Path $remoteServerSrc -DestinationPath $remoteServerDst -Force

    Write-Output "Remote server compressed successfully"
}

function ZipZedAndItsFriendsDebug {
    $items = @(
        ".\$CargoOutDir\zed_custom.pdb",
        ".\$CargoOutDir\cli.pdb",
        ".\$CargoOutDir\auto_update_helper.pdb",
        ".\$CargoOutDir\explorer_command_injector.pdb",
        ".\$CargoOutDir\remote_server.pdb"
    )

    Compress-Archive -Path $items -DestinationPath ".\$CargoOutDir\zed-custom-$env:RELEASE_VERSION-$env:ZED_RELEASE_CHANNEL.dbg.zip" -Force
}


function UploadToSentry {
    if (-not (Get-Command "sentry-cli" -ErrorAction SilentlyContinue)) {
        Write-Output "sentry-cli not found. skipping sentry upload."
        Write-Output "install with: 'winget install -e --id=Sentry.sentry-cli'"
        return
    }
    if (-not (Test-Path "env:SENTRY_AUTH_TOKEN")) {
        Write-Output "missing SENTRY_AUTH_TOKEN. skipping sentry upload."
        return
    }
    Write-Output "Uploading zed-custom debug symbols to sentry..."
    for ($i = 1; $i -le 3; $i++) {
        try {
            sentry-cli debug-files upload --include-sources --wait -p zed-custom -o zed-custom-dev $CargoOutDir
            break
        }
        catch {
            Write-Output "Sentry upload attempt $i failed: $_"
            if ($i -eq 3) {
                Write-Output "All sentry upload attempts failed"
                throw
            }
            Start-Sleep -Seconds 2
        }
    }
}

function MakeAppx {
    switch ($channel) {
        "stable" {
            $manifestFile = "$env:ZED_WORKSPACE\crates\explorer_command_injector\AppxManifest.xml"
        }
        "preview" {
            $manifestFile = "$env:ZED_WORKSPACE\crates\explorer_command_injector\AppxManifest-Preview.xml"
        }
        default {
            $manifestFile = "$env:ZED_WORKSPACE\crates\explorer_command_injector\AppxManifest-Nightly.xml"
        }
    }
    Copy-Item -Path "$manifestFile" -Destination "$innoDir\make_appx\AppxManifest.xml"
    # Find makeAppx.exe in Windows Kits
    $windowsKitsPath = "${env:ProgramFiles(x86)}\Windows Kits\10\bin"
    $makeAppx = Get-ChildItem -Path $windowsKitsPath -Filter "makeAppx.exe" -Recurse | Where-Object { $_.FullName -like "*\x64\*" } | Select-Object -First 1
    
    if ($makeAppx) {
        $sdkBinDir = Split-Path -Path $makeAppx.FullName
        Write-Host "🚀 Found Windows SDK tools at $sdkBinDir"
        $env:Path += ';' + $sdkBinDir
        & $makeAppx.FullName pack /d "$innoDir\make_appx" /p "$innoDir\zed_explorer_command_injector.appx" /nv
    } else {
        Write-Error "Could not find makeAppx.exe in $windowsKitsPath"
        exit 1
    }
}

function SignZedAndItsFriends {
    if (-not $env:CI -or $env:ZED_SIGNING_READY -ne "true") {
        return
    }

    $files = "$innoDir\zed-custom.exe,$innoDir\cli.exe,$innoDir\auto_update_helper.exe,$innoDir\zed_explorer_command_injector.dll,$innoDir\zed_explorer_command_injector.appx"
    & "$innoDir\sign.ps1" $files
}

function DownloadAMDGpuServices {
    # If you update the AGS SDK version, please also update the version in `crates/gpui/src/platform/windows/directx_renderer.rs`
    $url = "https://codeload.github.com/GPUOpen-LibrariesAndSDKs/AGS_SDK/zip/refs/tags/v6.3.0"
    $zipPath = ".\AGS_SDK_v6.3.0.zip"
    # Download the AGS SDK zip file
    Invoke-WebRequest -Uri $url -OutFile $zipPath
    # Extract the AGS SDK zip file
    Expand-Archive -Path $zipPath -DestinationPath "." -Force
}

function DownloadConpty {
    $url = "https://github.com/microsoft/terminal/releases/download/v1.23.13503.0/Microsoft.Windows.Console.ConPTY.1.23.251216003.nupkg"
    $zipPath = ".\conpty.zip"
    Invoke-WebRequest -Uri $url -OutFile $zipPath
    Expand-Archive -Path $zipPath -DestinationPath ".\conpty" -Force
}

function CollectFiles {
    Move-Item -Path "$innoDir\zed_explorer_command_injector.appx" -Destination "$innoDir\appx\zed_explorer_command_injector.appx" -Force
    Move-Item -Path "$innoDir\zed_explorer_command_injector.dll" -Destination "$innoDir\appx\zed_explorer_command_injector.dll" -Force
    Move-Item -Path "$innoDir\cli.exe" -Destination "$innoDir\bin\zed-custom.exe" -Force
    Move-Item -Path "$innoDir\zed.sh" -Destination "$innoDir\bin\zed-custom" -Force
    Move-Item -Path "$innoDir\auto_update_helper.exe" -Destination "$innoDir\tools\auto_update_helper.exe" -Force
    if($Architecture -eq "aarch64") {
        New-Item -Type Directory -Path "$innoDir\arm64" -Force
        Move-Item -Path ".\conpty\build\native\runtimes\arm64\OpenConsole.exe" -Destination "$innoDir\arm64\OpenConsole.exe" -Force
        Move-Item -Path ".\conpty\runtimes\win-arm64\native\conpty.dll" -Destination "$innoDir\conpty.dll" -Force
    }
    else {
        New-Item -Type Directory -Path "$innoDir\x64" -Force
        New-Item -Type Directory -Path "$innoDir\arm64" -Force
        Move-Item -Path ".\AGS_SDK-6.3.0\ags_lib\lib\amd_ags_x64.dll" -Destination "$innoDir\amd_ags_x64.dll" -Force
        Move-Item -Path ".\conpty\build\native\runtimes\x64\OpenConsole.exe" -Destination "$innoDir\x64\OpenConsole.exe" -Force
        Move-Item -Path ".\conpty\build\native\runtimes\arm64\OpenConsole.exe" -Destination "$innoDir\arm64\OpenConsole.exe" -Force
        Move-Item -Path ".\conpty\runtimes\win-x64\native\conpty.dll" -Destination "$innoDir\conpty.dll" -Force
    }
}

function BuildInstaller {
    $issFilePath = "$innoDir\zed.iss"
    switch ($channel) {
        "stable" {
            $appId = "{{2DB0DA96-CA55-49BB-AF4F-64AF36A86712}"
            $appIconName = "app-icon"
            $appName = "zed-custom"
            $appDisplayName = "zed-custom"
            $appSetupName = "zed-custom-$Architecture"
            # The mutex name here should match the mutex name in crates\zed-custom\src\zed-custom\windows_only_instance.rs
            $appMutex = "zed-custom-Stable-Instance-Mutex"
            $appExeName = "zed-custom"
            $regValueName = "zed-custom"
            $appUserId = "ZedIndustries.zed-custom"
            $appShellNameShort = "Z&ed"
            $appAppxFullName = "ZedIndustries.Zed_1.0.0.0_neutral__japxn1gcva8rg"
        }
        "preview" {
            $appId = "{{F70E4811-D0E2-4D88-AC99-D63752799F95}"
            $appIconName = "app-icon-preview"
            $appName = "zed-custom Preview"
            $appDisplayName = "zed-custom Preview"
            $appSetupName = "zed-custom-$Architecture"
            # The mutex name here should match the mutex name in crates\zed-custom\src\zed-custom\windows_only_instance.rs
            $appMutex = "zed-custom-Preview-Instance-Mutex"
            $appExeName = "zed-custom"
            $regValueName = "ZedPreview"
            $appUserId = "ZedIndustries.zed-custom.Preview"
            $appShellNameShort = "Z&ed Preview"
            $appAppxFullName = "ZedIndustries.zed-custom.Preview_1.0.0.0_neutral__japxn1gcva8rg"
        }
        "nightly" {
            $appId = "{{1BDB21D3-14E7-433C-843C-9C97382B2FE0}"
            $appIconName = "app-icon-nightly"
            $appName = "zed-custom Nightly"
            $appDisplayName = "zed-custom Nightly"
            $appSetupName = "zed-custom-$Architecture"
            # The mutex name here should match the mutex name in crates\zed-custom\src\zed-custom\windows_only_instance.rs
            $appMutex = "zed-custom-Nightly-Instance-Mutex"
            $appExeName = "zed-custom"
            $regValueName = "ZedNightly"
            $appUserId = "ZedIndustries.zed-custom.Nightly"
            $appShellNameShort = "Z&ed Editor Nightly"
            $appAppxFullName = "ZedIndustries.zed-custom.Nightly_1.0.0.0_neutral__japxn1gcva8rg"
        }
        "dev" {
            $appId = "{{8357632E-24A4-4F32-BA97-E575B4D1FE5D}"
            $appIconName = "app-icon-dev"
            $appName = "zed-custom Dev"
            $appDisplayName = "zed-custom Dev"
            $appSetupName = "zed-custom-$Architecture"
            # The mutex name here should match the mutex name in crates\zed-custom\src\zed-custom\windows_only_instance.rs
            $appMutex = "zed-custom-Dev-Instance-Mutex"
            $appExeName = "zed-custom"
            $regValueName = "ZedDev"
            $appUserId = "ZedIndustries.zed-custom.Dev"
            $appShellNameShort = "Z&ed Dev"
            $appAppxFullName = "ZedIndustries.zed-custom.Dev_1.0.0.0_neutral__japxn1gcva8rg"
        }
        default {
            Write-Error "can't bundle installer for $channel."
            exit 1
        }
    }

    # Windows runner 2022 default has iscc in PATH, https://github.com/actions/runner-images/blob/main/images/windows/Windows2022-Readme.md
    # Currently, we are using Windows 2022 runner.
    # Windows runner 2025 doesn't have iscc in PATH for now, https://github.com/actions/runner-images/issues/11228
    $innoSetupPath = "C:\Program Files (x86)\Inno Setup 6\ISCC.exe"

    $definitions = @{
        "AppId"          = $appId
        "AppIconName"    = $appIconName
        "OutputDir"      = "$env:ZED_WORKSPACE\target"
        "AppSetupName"   = $appSetupName
        "AppName"        = $appName
        "AppDisplayName" = $appDisplayName
        "RegValueName"   = $regValueName
        "AppMutex"       = $appMutex
        "AppExeName"     = $appExeName
        "ResourcesDir"   = "$innoDir"
        "ShellNameShort" = $appShellNameShort
        "AppUserId"      = $appUserId
        "Version"        = "$env:RELEASE_VERSION"
        "SourceDir"      = "$env:ZED_WORKSPACE"
        "AppxFullName"   = $appAppxFullName
    }

    $innoArgs = @()
    foreach ($key in $definitions.Keys) {
        $innoArgs += "/d$key=`"$($definitions[$key])`""
    }

    if($env:CI -and $env:ZED_SIGNING_READY -eq "true") {
        $innoSignScript = "$innoDir\sign.ps1"
        $innoArgs += "/sDefaultsign=`"powershell.exe -ExecutionPolicy Bypass -File `"`"$innoSignScript`"`" `"`$f`"`"`""
        $innoArgs += "/dSigningReady=true"
    }

    $innoArgs += $issFilePath

    # Execute Inno Setup
    Write-Host "🚀 Running Inno Setup: $innoSetupPath $innoArgs"
    $process = Start-Process -FilePath $innoSetupPath -ArgumentList $innoArgs -NoNewWindow -Wait -PassThru

    if ($process.ExitCode -eq 0) {
        Write-Host "✅ Inno Setup successfully compiled the installer"
        Write-Output "SETUP_PATH=target/$appSetupName.exe" >> $env:GITHUB_ENV
        $script:buildSuccess = $true
    }
    else {
        Write-Host "❌ Inno Setup failed: $($process.ExitCode)"
        $script:buildSuccess = $false
    }
}

ParseZedWorkspace
$innoDir = "$env:ZED_WORKSPACE\inno\$Architecture"
$debugArchive = "$CargoOutDir\zed-custom-$env:RELEASE_VERSION-$env:ZED_RELEASE_CHANNEL.dbg.zip"
$debugStoreKey = "$env:ZED_RELEASE_CHANNEL/zed-custom-$env:RELEASE_VERSION-$env:ZED_RELEASE_CHANNEL.dbg.zip"

CheckEnvironmentVariables
PrepareForBundle
GenerateLicenses
BuildZedAndItsFriends
BuildRemoteServer
MakeAppx
SignZedAndItsFriends
ZipZedAndItsFriendsDebug
DownloadAMDGpuServices
DownloadConpty
CollectFiles
BuildInstaller

if($env:CI) {
    UploadToSentry
}

if ($buildSuccess) {
    Write-Output "Build successful"
    if ($Install) {
        Write-Output "Installing zed-custom..."
        Start-Process -FilePath "$env:ZED_WORKSPACE/target/ZedEditorUserSetup-x64-$env:RELEASE_VERSION.exe"
    }
    exit 0
}
else {
    Write-Output "Build failed"
    exit 1
}
