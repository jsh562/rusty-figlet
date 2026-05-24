# Print an optspec for argparse to handle cmd's options that are independent of any subcommand.
function __fish_rusty_figlet_global_optspecs
	string join \n f/font= d/fontdir= w/width= t/terminal-width c/center l/left r/right x/font-default-justify k/kerning W/full-width S/force-smush s/smush o/overlap m/layout-mode= p/paragraph n/normal C/control-file= N/no-controlfile color= rainbow strict no-strict h/help V/version
end

function __fish_rusty_figlet_needs_command
	# Figure out if the current invocation already has a command.
	set -l cmd (commandline -opc)
	set -e cmd[1]
	argparse -s (__fish_rusty_figlet_global_optspecs) -- $cmd 2>/dev/null
	or return
	if set -q argv[1]
		# Also print the command, so this can be used to figure out what it is.
		echo $argv[1]
		return 1
	end
	return 0
end

function __fish_rusty_figlet_using_subcommand
	set -l cmd (__fish_rusty_figlet_needs_command)
	test -z "$cmd"
	and return 1
	contains -- $cmd[1] $argv
end

complete -c rusty-figlet -n "__fish_rusty_figlet_needs_command" -s f -l font -r
complete -c rusty-figlet -n "__fish_rusty_figlet_needs_command" -s d -l fontdir -r -F
complete -c rusty-figlet -n "__fish_rusty_figlet_needs_command" -s w -l width -r
complete -c rusty-figlet -n "__fish_rusty_figlet_needs_command" -s m -l layout-mode -r
complete -c rusty-figlet -n "__fish_rusty_figlet_needs_command" -s C -l control-file -r -F
complete -c rusty-figlet -n "__fish_rusty_figlet_needs_command" -l color -r -f -a "auto\t'Auto-detect from TTY status'
always\t'Always emit color (still suppressed by NO_COLOR per FR-032)'
never\t'Never emit color'"
complete -c rusty-figlet -n "__fish_rusty_figlet_needs_command" -s t -l terminal-width
complete -c rusty-figlet -n "__fish_rusty_figlet_needs_command" -s c -l center
complete -c rusty-figlet -n "__fish_rusty_figlet_needs_command" -s l -l left
complete -c rusty-figlet -n "__fish_rusty_figlet_needs_command" -s r -l right
complete -c rusty-figlet -n "__fish_rusty_figlet_needs_command" -s x -l font-default-justify
complete -c rusty-figlet -n "__fish_rusty_figlet_needs_command" -s k -l kerning
complete -c rusty-figlet -n "__fish_rusty_figlet_needs_command" -s W -l full-width
complete -c rusty-figlet -n "__fish_rusty_figlet_needs_command" -s S -l force-smush
complete -c rusty-figlet -n "__fish_rusty_figlet_needs_command" -s s -l smush
complete -c rusty-figlet -n "__fish_rusty_figlet_needs_command" -s o -l overlap
complete -c rusty-figlet -n "__fish_rusty_figlet_needs_command" -s p -l paragraph
complete -c rusty-figlet -n "__fish_rusty_figlet_needs_command" -s n -l normal
complete -c rusty-figlet -n "__fish_rusty_figlet_needs_command" -s N -l no-controlfile
complete -c rusty-figlet -n "__fish_rusty_figlet_needs_command" -l rainbow
complete -c rusty-figlet -n "__fish_rusty_figlet_needs_command" -l strict
complete -c rusty-figlet -n "__fish_rusty_figlet_needs_command" -l no-strict
complete -c rusty-figlet -n "__fish_rusty_figlet_needs_command" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c rusty-figlet -n "__fish_rusty_figlet_needs_command" -s V -l version -d 'Print version'
complete -c rusty-figlet -n "__fish_rusty_figlet_needs_command" -a "completions" -d 'Emit a shell-completion script for the named shell to stdout (FR-060 + US7 AS1). Generates the script via `clap_complete::generate` against the binary\'s own CLI surface'
complete -c rusty-figlet -n "__fish_rusty_figlet_needs_command" -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c rusty-figlet -n "__fish_rusty_figlet_using_subcommand completions" -s h -l help -d 'Print help'
complete -c rusty-figlet -n "__fish_rusty_figlet_using_subcommand help; and not __fish_seen_subcommand_from completions help" -f -a "completions" -d 'Emit a shell-completion script for the named shell to stdout (FR-060 + US7 AS1). Generates the script via `clap_complete::generate` against the binary\'s own CLI surface'
complete -c rusty-figlet -n "__fish_rusty_figlet_using_subcommand help; and not __fish_seen_subcommand_from completions help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
