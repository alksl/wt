# Shell integration for wt. Load it with: eval "$(wt init zsh)"

# `wt switch` prints the worktree path; cd to it here, since a child process can't. Its help is
# printed as is.
function wt() {
    case "$1" in
        switch|sw)
            if (( ${@[(I)(-h|--help)]} )); then
                command wt "$@"
                return
            fi
            local target
            target=$(command wt "$@") || return
            builtin cd -- "$target"
            ;;
        *)
            command wt "$@"
            ;;
    esac
}

# fzf's `wt sw **<TAB>` / `wt new **<TAB>` / `wt rm **<TAB>` completion. `sw -c` creates, so it
# offers what `new` does.
function _fzf_complete_wt() {
    local args="$*"
    if [[ $args == "wt new "* || $args == "wt "(switch|sw)" "(*" "|)(-c|--create)" "* ]]; then
        _fzf_complete "--reverse --tiebreak=begin" "$@" < <(
            command wt complete new 2>/dev/null
        )
    elif [[ $args == "wt switch"* || $args == "wt sw"* ]]; then
        _fzf_complete "--reverse --tiebreak=begin" "$@" < <(
            command wt complete switch 2>/dev/null
        )
    elif [[ $args == "wt remove"* || $args == "wt rm"* ]]; then
        _fzf_complete "--reverse --tiebreak=begin" "$@" < <(
            command wt complete remove 2>/dev/null
        )
    else
        eval "zle ${fzf_default_completion:-expand-or-complete}"
    fi
}

# Candidates may carry a tab and where the branch lives (`fix-typo<TAB>origin`); insert the name.
function _fzf_complete_wt_post() {
    cut -f1
}
