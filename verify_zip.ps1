Add-Type -AssemblyName System.IO.Compression.FileSystem
$zip = [System.IO.Compression.ZipFile]::OpenRead("$PSScriptRoot\artifacts\os\polymera-os-avengers.zip")
Write-Host "=== Polymera OS Archive Contents ===" -ForegroundColor Cyan
Write-Host ("{0,-55} {1,10}" -f "FILE", "SIZE (bytes)")
Write-Host ("-" * 67)
foreach ($entry in $zip.Entries) {
    Write-Host ("{0,-55} {1,10}" -f $entry.FullName, $entry.Length)
}
Write-Host ""
Write-Host ("Total entries: {0}" -f $zip.Entries.Count) -ForegroundColor Green
Write-Host ("Archive size:  {0:N0} bytes" -f (Get-Item "$PSScriptRoot\artifacts\os\polymera-os-avengers.zip").Length) -ForegroundColor Green
$zip.Dispose()
