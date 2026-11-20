<#
.SYNOPSIS
    src/ 以下に分割されたテンプレートを、提出用の単一ファイルにまとめる。

.DESCRIPTION
    src/engine/mod.rs は `// BUNDLE:<KEY>-BEGIN` 〜 `// BUNDLE:<KEY>-END` で
    各探索アルゴリズム (SA / HILLCLIMB / GREEDY / BEAM / NAIVE) の mod 宣言と
    solve() 内の対応する match 分岐を囲んである。
    -Algorithms で選ばなかったアルゴリズムはその区間ごと削除し、
    main.rs から辿れる `mod xxx;` 宣言 (xxx.rs / xxx/mod.rs) を再帰的に
    インライン展開して1ファイルに結合する。マーカーはどのファイルに書いてもよい。

.PARAMETER Algorithms
    含めるアルゴリズム。省略時は全て含める（絞り込みなし）。
    sa, hillclimb, greedy, beam, naive から複数選択可。

.PARAMETER OutFile
    出力先ファイル名。既定は submit.rs。

.EXAMPLE
    ./tools/bundle.ps1 -Algorithms sa
    SA (焼きなまし) だけを含めた submit.rs を生成する。

.EXAMPLE
    ./tools/bundle.ps1 -Algorithms sa,beam -OutFile submit_sa_beam.rs
#>
param(
    [ValidateSet("sa", "hillclimb", "greedy", "beam", "naive")]
    [string[]]$Algorithms = @("sa", "hillclimb", "greedy", "beam", "naive"),
    [string]$OutFile = "submit.rs"
)

$ErrorActionPreference = "Stop"

$repoRoot = Split-Path -Parent $PSScriptRoot
$srcDir = Join-Path $repoRoot "src"

$selectedKeys = $Algorithms | ForEach-Object { $_.ToUpper() }

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

# `mod foo;` を `mod foo { <foo.rs or foo/mod.rs の中身> }` に再帰的に置き換える。
# $ModDir は、そのファイル内の子モジュールを探すディレクトリ。
function Expand-Module {
    param([string]$FilePath, [string]$ModDir)

    $lines = Expand-BundleMarkers -Lines (Get-Content $FilePath) -SelectedKeys $selectedKeys
    $result = New-Object System.Collections.Generic.List[string]
    foreach ($line in $lines) {
        if ($line -match '^(\s*)((?:pub(?:\([^)]*\))?\s+)?mod\s+(\w+));\s*$') {
            $indent = $Matches[1]
            $decl = $Matches[2]
            $modName = $Matches[3]
            $flat = Join-Path $ModDir "$modName.rs"
            $nested = Join-Path (Join-Path $ModDir $modName) "mod.rs"
            if (Test-Path $flat) {
                $childFile = $flat
            }
            elseif (Test-Path $nested) {
                $childFile = $nested
            }
            else {
                throw "Module '$modName' declared in $FilePath was not found ($flat or $nested)."
            }
            $childLines = Expand-Module -FilePath $childFile -ModDir (Join-Path $ModDir $modName)
            $result.Add("$indent$decl {")
            foreach ($childLine in $childLines) {
                $result.Add("$indent    $childLine")
            }
            $result.Add("$indent}")
        }
        else {
            $result.Add($line)
        }
    }
    return , $result
}

$output = Expand-Module -FilePath (Join-Path $srcDir "main.rs") -ModDir $srcDir

$outPath = Join-Path $srcDir $OutFile
Set-Content -Path $outPath -Value $output -Encoding utf8NoBOM

$rustfmt = Get-Command rustfmt -ErrorAction SilentlyContinue
if ($rustfmt) {
    & rustfmt $outPath
}

Write-Host "Bundled [$($Algorithms -join ', ')] -> $outPath"
