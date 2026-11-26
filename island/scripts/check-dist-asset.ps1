$c = Get-Content 'X:\folder_ai\Hime-Bloom\dist\assets\main-yUI34itx.js' -Raw
$idx = $c.IndexOf('hime-panel')
if ($idx -ge 0) {
    Write-Output $c.Substring($idx - 350, 450)
}
