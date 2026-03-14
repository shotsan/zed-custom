[CmdletBinding()]
Param(
    [Parameter()][string]$Architecture
)

# Based on the template in: https://docs.digitalocean.com/reference/api/spaces-api/
$ErrorActionPreference = "Stop"
. "$PSScriptRoot\lib\blob-store.ps1"
. "$PSScriptRoot\lib\workspace.ps1"

ParseZedWorkspace
Write-Host "Uploading nightly for target: $target"

$bucketName = "zed-custom-nightly-host"
$releaseVersion = & "$PSScriptRoot\get-crate-version.ps1" zed-custom
$version = "$releaseVersion+nightly.$env:GITHUB_RUN_NUMBER.$env:GITHUB_SHA"

$remoteServerFiles = Get-ChildItem -Path "target" -Filter "zed-custom-remote-server-windows-*.zip" -Recurse -File -ErrorAction SilentlyContinue

foreach ($file in $remoteServerFiles) {
    UploadToBlobStore -BucketName $bucketName -FileToUpload $file.FullName -BlobStoreKey "nightly/$($file.Name)"
    UploadToBlobStore -BucketName $bucketName -FileToUpload $file.FullName -BlobStoreKey "$version/$($file.Name)"
    Remove-Item -Path $file.FullName -ErrorAction SilentlyContinue
}

UploadToBlobStore -BucketName $bucketName -FileToUpload "target/zed-custom-$Architecture.exe" -BlobStoreKey "nightly/zed-custom-$Architecture.exe"
UploadToBlobStore -BucketName $bucketName -FileToUpload "target/zed-custom-$Architecture.exe" -BlobStoreKey "$version/zed-custom-$Architecture.exe"

Remove-Item -Path "target/zed-custom-$Architecture.exe" -ErrorAction SilentlyContinue

$version | Out-File -FilePath "target/latest-sha" -NoNewline
UploadToBlobStore -BucketName $bucketName -FileToUpload "target/latest-sha" -BlobStoreKey "nightly/latest-sha-windows"
Remove-Item -Path "target/latest-sha" -ErrorAction SilentlyContinue
