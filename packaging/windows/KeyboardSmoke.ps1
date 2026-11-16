function New-KeyboardCase([string]$Directory, [string]$Language, [string]$View, [string]$ProjectName) {
    $null = New-Item -ItemType Directory -Path $Directory
    $project = Join-Path $Directory $ProjectName
    Copy-Item -LiteralPath 'fixtures/valid/lan.json' -Destination $project
    $preferences = Get-Content -Raw -Encoding UTF8 -LiteralPath 'fixtures/valid/preferences-v2.json' | ConvertFrom-Json
    $preferences.language = $Language
    $preferences.display.representation = $View
    $preferences.display.scale_percent = 150
    $preferences.display.window_width = 1600
    $preferences.display.window_height = 1050
    $settings = Join-Path $Directory 'settings.json'
    [IO.File]::WriteAllText($settings, ($preferences | ConvertTo-Json -Depth 20), [Text.UTF8Encoding]::new($false))
    [pscustomobject]@{ Directory = $Directory; Project = $project; Language = $Language; View = $View }
}

function Wait-KeyboardProbe([string]$Description, [scriptblock]$Probe) {
    $deadline = [Diagnostics.Stopwatch]::StartNew()
    do {
        try {
            $value = & $Probe
            if ($value) { return $value }
        } catch [System.Windows.Automation.ElementNotAvailableException] {
            # The workbench can replace its automation nodes between frames.
        }
        Start-Sleep -Milliseconds 75
    } while ($deadline.Elapsed.TotalSeconds -lt 20)
    throw "Timed out waiting for $Description"
}

function Test-DesktopKeyboard($Process, $Case, $Catalog) {
    $language = $Case.Language
    $view = $Case.View
    $description = "$language/$view"
    $handle = $Process.MainWindowHandle
    $null = Wait-KeyboardProbe "$description active window" {
        [SinesiShellSmoke]::GetForegroundWindow() -eq $handle
    }
    $window = [System.Windows.Automation.AutomationElement]::FromHandle($handle)

    function Send-ProjectKey([string]$Key) {
        if ($Process.HasExited -or [SinesiShellSmoke]::GetForegroundWindow() -ne $handle) {
            throw "$description lost the installed application's foreground window"
        }
        [System.Windows.Forms.SendKeys]::SendWait($Key)
    }

    function Find-ProjectLabel([string]$Label) {
        $condition = [System.Windows.Automation.PropertyCondition]::new(
            [System.Windows.Automation.AutomationElement]::NameProperty, $Label
        )
        $window.FindFirst([System.Windows.Automation.TreeScope]::Descendants, $condition)
    }

    function Invoke-ProjectControl([string]$Label) {
        $null = Wait-KeyboardProbe "$description control '$Label'" { Find-ProjectLabel $Label }
        for ($tab = 0; $tab -lt 160; $tab++) {
            try {
                $focused = [System.Windows.Automation.AutomationElement]::FocusedElement
                $isTarget = $focused -and $focused.Current.ProcessId -eq $Process.Id -and $focused.Current.Name -ceq $Label
            } catch [System.Windows.Automation.ElementNotAvailableException] {
                $isTarget = $false
            }
            if ($isTarget) {
                Send-ProjectKey '{ENTER}'
                Start-Sleep -Milliseconds 150
                return
            }
            Send-ProjectKey '{TAB}'
            Start-Sleep -Milliseconds 75
        }
        throw "$description did not expose keyboard focus on '$Label'"
    }

    $original = Get-Content -Raw -Encoding UTF8 -LiteralPath $Case.Project | ConvertFrom-Json
    $deviceCount = $original.devices.Count + 1
    Invoke-ProjectControl $Catalog.add_notebook.$language
    $canvasLabel = $Catalog.canvas_device.$language
    $newDevice = "$($Catalog.notebook.$language) $deviceCount"
    $null = Wait-KeyboardProbe "$description new laptop" { Find-ProjectLabel ($canvasLabel.Replace('{label}', $newDevice)) }
    Send-ProjectKey '^s'
    $null = Wait-KeyboardProbe "$description project saved by Ctrl+S" {
        $saved = Get-Content -Raw -Encoding UTF8 -LiteralPath $Case.Project | ConvertFrom-Json
        $saved.devices.Count -eq $deviceCount
    }
    $saved = Get-Content -Raw -Encoding UTF8 -LiteralPath $Case.Project | ConvertFrom-Json
    if (($saved.links | ConvertTo-Json -Depth 20 -Compress) -cne ($original.links | ConvertTo-Json -Depth 20 -Compress)) {
        throw "$description changed existing cables during save"
    }
    $savedHash = (Get-FileHash -LiteralPath $Case.Project).Hash
    Send-ProjectKey '{F3}'
    $null = Wait-KeyboardProbe "$description simulation mode" { Find-ProjectLabel $Catalog.step.$language }
    Invoke-ProjectControl ($canvasLabel.Replace('{label}', $original.devices[0].label))
    Invoke-ProjectControl $Catalog.send_ping.$language
    Invoke-ProjectControl $Catalog.resume.$language
    Invoke-ProjectControl $Catalog.exchanges.$language
    $null = Wait-KeyboardProbe "$description ICMP reply" { Find-ProjectLabel 'icmp.echo.reply' }
    if ((Get-FileHash -LiteralPath $Case.Project).Hash -ne $savedHash) {
        throw "$description simulation modified the saved project"
    }
    Write-Output "OK - $description at 150%: associated window, UI Automation focus, Ctrl+S, ICMP reply"
}
