# Shell integration for wt. Load it with: eval "$(wt init zsh)"

# `wt switch` prints the worktree path; cd to it here, since a child process can't.
function wt() {
    case "$1" in
        switch|sw)
            local target
            target=$(command wt "$@") || return
            builtin cd -- "$target"
            ;;
        *)
            command wt "$@"
            ;;
    esac
}

# fzf's `wt sw **<TAB>` / `wt rm **<TAB>` completion.
function _fzf_complete_wt() {
    local args="$*"
    if [[ $args == "wt switch"* || $args == "wt sw"* ]]; then
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

function _fzf_complete_wt_post() {
    cat
}
