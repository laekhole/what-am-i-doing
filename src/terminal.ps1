# Read-only UI Automation. No input, tab switching, scrolling or remote commands.
$ErrorActionPreference = 'Stop'
[Console]::OutputEncoding = [Text.UTF8Encoding]::new($false)
Add-Type -AssemblyName UIAutomationClient
Add-Type -AssemblyName UIAutomationTypes
$screens = @()
$failed = $false
$root = [Windows.Automation.AutomationElement]::RootElement
$windows = $root.FindAll([Windows.Automation.TreeScope]::Children, [Windows.Automation.Condition]::TrueCondition)
foreach ($window in $windows) {
    if ($screens.Count -ge 16) { break }
    try {
        $process = Get-Process -Id $window.Current.ProcessId -ErrorAction Stop
        if ($process.ProcessName -notin @('WindowsTerminal', 'conhost', 'OpenConsole', 'powershell', 'pwsh')) { continue }
        if ($window.Current.IsOffscreen) { continue }
        $owner = '{0}:{1}:{2}' -f $process.Id, $process.StartTime.ToUniversalTime().Ticks, $window.Current.NativeWindowHandle
        $elements = $window.FindAll([Windows.Automation.TreeScope]::Descendants, [Windows.Automation.Condition]::TrueCondition)
        $selected = ''
        $selectedElement = $null
        foreach ($element in $elements) {
            if ($element.Current.ControlType -ne [Windows.Automation.ControlType]::TabItem) { continue }
            $selection = $null
            if ($element.TryGetCurrentPattern([Windows.Automation.SelectionItemPattern]::Pattern, [ref]$selection) -and $selection.Current.IsSelected) {
                $selected = $element.GetRuntimeId() -join '.'
                $selectedElement = $element
            }
        }
        foreach ($element in $elements) {
            if ($screens.Count -ge 16) { break }
            # Tab labels also expose TextPattern; only read terminal documents.
            if ($element.Current.ClassName -ne 'TermControl' -and
                $element.Current.ControlType -ne [Windows.Automation.ControlType]::Document) { continue }
            if ($element.Current.IsOffscreen) { continue }
            $pattern = $null
            if (-not $element.TryGetCurrentPattern([Windows.Automation.TextPattern]::Pattern, [ref]$pattern)) { continue }
            $body = ''
            foreach ($range in $pattern.GetVisibleRanges()) {
                if ($body.Length -ge 16000) { break }
                $body += $range.GetText(16000 - $body.Length)
            }
            if ($null -ne $selectedElement -and -not $selectedElement.GetCurrentPattern([Windows.Automation.SelectionItemPattern]::Pattern).Current.IsSelected) { continue }
            $screens += @{
                key = '{0}:{1}:{2}' -f $owner, $selected, ($element.GetRuntimeId() -join '.')
                title = $window.Current.Name
                body = $body
            }
        }
    } catch { $failed = $true }
}
ConvertTo-Json -InputObject @{screens = $screens; failed = $failed} -Depth 4 -Compress
