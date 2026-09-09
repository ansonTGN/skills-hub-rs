param(
  [string]$Bin = ".\target\release\skills-hub-rs.exe"
)
$ErrorActionPreference = "Stop"
$Root = Join-Path $env:TEMP ("skills-hub-rs-smoke-" + [guid]::NewGuid().ToString())
$Hub = Join-Path $Root "hub"
$Custom = Join-Path $Root "custom-tool\skills"
$Project = Join-Path $Root "project"
New-Item -ItemType Directory -Force -Path $Project | Out-Null
try {
  Write-Host "[1/8] init"
  & $Bin --home $Hub init

  Write-Host "[2/8] tool catalog"
  $tools = (& $Bin --home $Hub tools --json) | ConvertFrom-Json
  if ($tools.Count -lt 47) { throw "Expected at least 47 tools; got $($tools.Count)" }

  Write-Host "[3/8] custom tool"
  & $Bin --home $Hub custom-tool add local_test --label "Local Test" --global-dir $Custom --project-dir ".agent/skills" --mode copy

  Write-Host "[4/8] install + sync"
  & $Bin --home $Hub add-local examples\demo-skill --tag smoke --tool local_test --mode copy
  if (!(Test-Path (Join-Path $Custom "demo-skill\SKILL.md"))) { throw "Global sync missing" }

  Write-Host "[5/8] disable/enable"
  & $Bin --home $Hub disable demo-skill
  if (Test-Path (Join-Path $Custom "demo-skill")) { throw "Disable did not remove target" }
  & $Bin --home $Hub enable demo-skill

  Write-Host "[6/8] project sync"
  & $Bin --home $Hub sync demo-skill --tool local_test --scope project --project $Project --mode copy
  if (!(Test-Path (Join-Path $Project ".agent\skills\demo-skill\SKILL.md"))) { throw "Project sync missing" }

  Write-Host "[7/8] recycle/restore"
  $line = & $Bin --home $Hub remove demo-skill
  $rid = ($line -replace '^.*recycle bin: ', '').Trim()
  if (!$rid) { throw "Recycle id missing" }
  & $Bin --home $Hub recycle restore $rid
  if (!(Test-Path (Join-Path $Hub "skills\demo-skill\SKILL.md"))) { throw "Restore missing" }

  Write-Host "[8/8] final list"
  & $Bin --home $Hub list
  Write-Host "SMOKE TEST OK"
} finally {
  Remove-Item -Recurse -Force -ErrorAction SilentlyContinue $Root
}
