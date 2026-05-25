
using namespace System.Management.Automation
using namespace System.Management.Automation.Language

Register-ArgumentCompleter -Native -CommandName 'rusty-figlet' -ScriptBlock {
    param($wordToComplete, $commandAst, $cursorPosition)

    $commandElements = $commandAst.CommandElements
    $command = @(
        'rusty-figlet'
        for ($i = 1; $i -lt $commandElements.Count; $i++) {
            $element = $commandElements[$i]
            if ($element -isnot [StringConstantExpressionAst] -or
                $element.StringConstantType -ne [StringConstantType]::BareWord -or
                $element.Value.StartsWith('-') -or
                $element.Value -eq $wordToComplete) {
                break
        }
        $element.Value
    }) -join ';'

    $completions = @(switch ($command) {
        'rusty-figlet' {
            [CompletionResult]::new('-f', '-f', [CompletionResultType]::ParameterName, 'f')
            [CompletionResult]::new('--font', '--font', [CompletionResultType]::ParameterName, 'font')
            [CompletionResult]::new('-d', '-d', [CompletionResultType]::ParameterName, 'd')
            [CompletionResult]::new('--fontdir', '--fontdir', [CompletionResultType]::ParameterName, 'fontdir')
            [CompletionResult]::new('-w', '-w', [CompletionResultType]::ParameterName, 'w')
            [CompletionResult]::new('--width', '--width', [CompletionResultType]::ParameterName, 'width')
            [CompletionResult]::new('-m', '-m', [CompletionResultType]::ParameterName, 'm')
            [CompletionResult]::new('--layout-mode', '--layout-mode', [CompletionResultType]::ParameterName, 'layout-mode')
            [CompletionResult]::new('-C', '-C ', [CompletionResultType]::ParameterName, 'C')
            [CompletionResult]::new('--control-file', '--control-file', [CompletionResultType]::ParameterName, 'control-file')
            [CompletionResult]::new('--color', '--color', [CompletionResultType]::ParameterName, 'color')
            [CompletionResult]::new('-F', '-F ', [CompletionResultType]::ParameterName, 'Toilet-compatible filter chain (`-F filter1:filter2:...`)')
            [CompletionResult]::new('--filter', '--filter', [CompletionResultType]::ParameterName, 'Toilet-compatible filter chain (`-F filter1:filter2:...`)')
            [CompletionResult]::new('-E', '-E ', [CompletionResultType]::ParameterName, 'Export the rendered banner as `html`, `irc`, or `svg` (E012 US2 — FR-005, T061). Gated by any `output-*` leaf')
            [CompletionResult]::new('--export', '--export', [CompletionResultType]::ParameterName, 'Export the rendered banner as `html`, `irc`, or `svg` (E012 US2 — FR-005, T061). Gated by any `output-*` leaf')
            [CompletionResult]::new('--background', '--background', [CompletionResultType]::ParameterName, 'Background color spec — `<name>` (one of the 16 ANSI colors) or `#RRGGBB` (E012 US7 — SC-007, T063)')
            [CompletionResult]::new('-t', '-t', [CompletionResultType]::ParameterName, 't')
            [CompletionResult]::new('--terminal-width', '--terminal-width', [CompletionResultType]::ParameterName, 'terminal-width')
            [CompletionResult]::new('-c', '-c', [CompletionResultType]::ParameterName, 'c')
            [CompletionResult]::new('--center', '--center', [CompletionResultType]::ParameterName, 'center')
            [CompletionResult]::new('-l', '-l', [CompletionResultType]::ParameterName, 'l')
            [CompletionResult]::new('--left', '--left', [CompletionResultType]::ParameterName, 'left')
            [CompletionResult]::new('-r', '-r', [CompletionResultType]::ParameterName, 'r')
            [CompletionResult]::new('--right', '--right', [CompletionResultType]::ParameterName, 'right')
            [CompletionResult]::new('-x', '-x', [CompletionResultType]::ParameterName, 'x')
            [CompletionResult]::new('--font-default-justify', '--font-default-justify', [CompletionResultType]::ParameterName, 'font-default-justify')
            [CompletionResult]::new('-k', '-k', [CompletionResultType]::ParameterName, 'k')
            [CompletionResult]::new('--kerning', '--kerning', [CompletionResultType]::ParameterName, 'kerning')
            [CompletionResult]::new('-W', '-W ', [CompletionResultType]::ParameterName, 'W')
            [CompletionResult]::new('--full-width', '--full-width', [CompletionResultType]::ParameterName, 'full-width')
            [CompletionResult]::new('-S', '-S ', [CompletionResultType]::ParameterName, 'S')
            [CompletionResult]::new('--force-smush', '--force-smush', [CompletionResultType]::ParameterName, 'force-smush')
            [CompletionResult]::new('-s', '-s', [CompletionResultType]::ParameterName, 's')
            [CompletionResult]::new('--smush', '--smush', [CompletionResultType]::ParameterName, 'smush')
            [CompletionResult]::new('-o', '-o', [CompletionResultType]::ParameterName, 'o')
            [CompletionResult]::new('--overlap', '--overlap', [CompletionResultType]::ParameterName, 'overlap')
            [CompletionResult]::new('-p', '-p', [CompletionResultType]::ParameterName, 'p')
            [CompletionResult]::new('--paragraph', '--paragraph', [CompletionResultType]::ParameterName, 'paragraph')
            [CompletionResult]::new('-n', '-n', [CompletionResultType]::ParameterName, 'n')
            [CompletionResult]::new('--normal', '--normal', [CompletionResultType]::ParameterName, 'normal')
            [CompletionResult]::new('-N', '-N ', [CompletionResultType]::ParameterName, 'N')
            [CompletionResult]::new('--no-controlfile', '--no-controlfile', [CompletionResultType]::ParameterName, 'no-controlfile')
            [CompletionResult]::new('--rainbow', '--rainbow', [CompletionResultType]::ParameterName, 'rainbow')
            [CompletionResult]::new('--truecolor', '--truecolor', [CompletionResultType]::ParameterName, 'Force 24-bit truecolor SGR (E012 US4 — FR-008, T062)')
            [CompletionResult]::new('--ansi256', '--ansi256', [CompletionResultType]::ParameterName, 'Force 256-color SGR (E012 US4 — FR-009, T062)')
            [CompletionResult]::new('--no-downgrade-warning', '--no-downgrade-warning', [CompletionResultType]::ParameterName, 'Suppress the one-time downgrade-warning stderr line (E012 US4 — FR-029, T062)')
            [CompletionResult]::new('--warn-irc-strip', '--warn-irc-strip', [CompletionResultType]::ParameterName, 'Warn when IRC-format export strips a non-printable byte (E012 US2 — FR-015 ergonomics, T061)')
            [CompletionResult]::new('--strict', '--strict', [CompletionResultType]::ParameterName, 'strict')
            [CompletionResult]::new('--no-strict', '--no-strict', [CompletionResultType]::ParameterName, 'no-strict')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help (see more with ''--help'')')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help (see more with ''--help'')')
            [CompletionResult]::new('-V', '-V ', [CompletionResultType]::ParameterName, 'Print version')
            [CompletionResult]::new('--version', '--version', [CompletionResultType]::ParameterName, 'Print version')
            [CompletionResult]::new('completions', 'completions', [CompletionResultType]::ParameterValue, 'Emit a shell-completion script for the named shell to stdout (FR-060 + US7 AS1). Generates the script via `clap_complete::generate` against the binary''s own CLI surface')
            [CompletionResult]::new('help', 'help', [CompletionResultType]::ParameterValue, 'Print this message or the help of the given subcommand(s)')
            break
        }
        'rusty-figlet;completions' {
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            break
        }
        'rusty-figlet;help' {
            [CompletionResult]::new('completions', 'completions', [CompletionResultType]::ParameterValue, 'Emit a shell-completion script for the named shell to stdout (FR-060 + US7 AS1). Generates the script via `clap_complete::generate` against the binary''s own CLI surface')
            [CompletionResult]::new('help', 'help', [CompletionResultType]::ParameterValue, 'Print this message or the help of the given subcommand(s)')
            break
        }
        'rusty-figlet;help;completions' {
            break
        }
        'rusty-figlet;help;help' {
            break
        }
    })

    $completions.Where{ $_.CompletionText -like "$wordToComplete*" } |
        Sort-Object -Property ListItemText
}
