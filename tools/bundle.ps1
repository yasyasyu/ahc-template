<#
.SYNOPSIS
    src/ 以下に分割されたテンプレートを、提出用の単一ファイルにまとめる。

.DESCRIPTION
    src/main.rs は `// BUNDLE:<KEY>-BEGIN` 〜 `// BUNDLE:<KEY>-END` で
    各探索アルゴリズム (SA / HILLCLIMB / GREEDY / BEAM) の mod 宣言と
    solve() 内の対応する match 分岐を囲んである。
    -Algorithms で選ばなかったアルゴリズムはその区間ごと削除し、
    残った mod 宣言はファイル内容でインライン展開して1ファイルに結合する。
    util / problem / options は常に含める。

.PARAMETER Algorithms
    含めるアルゴリズム。省略時は全て含める（絞り込みなし）。
    sa, hillclimb, greedy, beam から複数選択可。

.PARAMETER OutFile
    出力先ファイル名。既定は submit.rs。

.EXAMPLE
    ./tools/bundle.ps1 -Algorithms sa
    SA (焼きなまし) だけを含めた submit.rs を生成する。

.EXAMPLE
    ./tools/bundle.ps1 -Algorithms sa,beam -OutFile submit_sa_beam.rs
#>
param(
    [ValidateSet("sa", "hillclimb", "greedy", "beam")]
    [string[]]$Algorithms = @("sa", "hillclimb", "greedy", "beam"),
    [string]$OutFile = "submit.rs"
)

$ErrorActionPreference = "Stop"

$repoRoot = Split-Path -Parent $PSScriptRoot
$srcDir = Join-Path $repoRoot "src"

$selectedKeys = $Algorithms | ForEach-Object { $_.ToUpper() }
$fileForMod = @{
    util       = "util.rs"
    problem    = "problem.rs"
    options    = "options.rs"
    annealing  = "annealing.rs"
    hill_climb = "hill_climb.rs"
    greedy     = "greedy.rs"
    beam       = "beam.rs"
}

function Expand-BundleMarkers {
    param([string[]]$Lines, [string[]]$SelectedKeys)

    $result = New-Object System.Collections.Generic.List[string]
    $skip = $false
    $removedAny = $false

    foreach ($line in $Lines) {
        if ($line -match '^\s*// BUNDLE:(\w+)-BEGIN\s*$') {
            $key = $Matches[1]
            if ($SelectedKeys -contains $key) {
                $skip = $false
            }
            else {
                $skip = $true
                $removedAny = $true
            }
            continue
        }
        if ($line -match '^\s*// BUNDLE:(\w+)-END\s*$') {
            $skip = $false
            continue
        }
        if ($line -match '^\s*// BUNDLE:FALLBACK\s*$') {
            if ($removedAny) {
                $result.Add('        _ => unreachable!("selected solver was not included in this bundle"),')
            }
            continue
        }
        if (-not $skip) {
            $result.Add($line)
        }
    }

    return , $result
}

$mainLines = Get-Content (Join-Path $srcDir "main.rs")
$filtered = Expand-BundleMarkers -Lines $mainLines -SelectedKeys $selectedKeys

$output = New-Object System.Collections.Generic.List[string]
foreach ($line in $filtered) {
    if ($line -match '^mod\s+(\w+);\s*$') {
        $modName = $Matches[1]
        $fileName = $fileForMod[$modName]
        if (-not $fileName) {
            throw "Unknown module '$modName' referenced in main.rs — add it to `$fileForMod in bundle.ps1."
        }
        $modContent = Get-Content (Join-Path $srcDir $fileName)
        $output.Add("mod $modName {")
        foreach ($modLine in $modContent) {
            $output.Add("    $modLine")
        }
        $output.Add("}")
    }
    else {
        $output.Add($line)
    }
}

$outPath = Join-Path $repoRoot $OutFile
Set-Content -Path $outPath -Value $output -Encoding utf8NoBOM

$rustfmt = Get-Command rustfmt -ErrorAction SilentlyContinue
if ($rustfmt) {
    & rustfmt $outPath
}

Write-Host "Bundled [$($Algorithms -join ', ')] -> $outPath"
