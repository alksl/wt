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

# fzf's `wt sw **<TAB>` / `wt new **<TAB>` / `wt rm **<TAB>` / `wt exec **<TAB>` completion. `sw -c`
# creates, so it offers what `new` does. Past `wt exec`'s `--` is the command, completed as usual.
function _fzf_complete_wt() {
    local args="$*"
    if [[ $args == "wt new "* || $args == "wt "(switch|sw)" "(*" "|)(-c|--create)" "* ]]; then
        _fzf_complete "--reverse --tiebreak=begin" "$@" < <(
            command wt complete new 2>/dev/null
        )
    elif [[ $args == "wt switch"* || $args == "wt sw"* ]]; then
        local -a words=(${(z)1})
        words=(${words[$((${words[(i)(sw|switch)]} + 1)),-1]})
        _fzf_complete "--reverse --tiebreak=begin" "$@" < <(
            command wt complete switch $words 2>/dev/null
        )
        # A project picked from outside a repo: offer its branches straight away. Esc leaves the
        # project alone, which switches to its main checkout.
        local picked=${${LBUFFER#$1}%% }
        if (( ! $#words )) && [[ $LBUFFER == "$1"?* && $picked != *' '* ]]; then
            local branches
            branches=$(command wt complete switch $picked 2>/dev/null)
            if [[ -n $branches ]]; then
                prefix= _fzf_complete "--reverse --tiebreak=begin" "$LBUFFER" <<< $branches
            fi
        fi
    elif [[ $args == "wt exec "* && $args != *" -- "* ]]; then
        _fzf_complete "--reverse --tiebreak=begin" "$@" < <(
            command wt complete exec 2>/dev/null
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

# Plain `wt sw <TAB>`, for when compinit has run: the same candidates as fzf's, without the picker.
# `local`/`origin` show as descriptions. Everything else completes as zsh would without this.
function _wt() {
    if (( CURRENT < 3 )) || [[ ${words[2]} != (sw|switch) ]]; then
        _default
        return
    fi
    local -a typed=(${${words[3,CURRENT-1]}:#-*}) candidates
    candidates=(${(f)"$(command wt complete switch ${(Q)typed} 2>/dev/null)"})
    candidates=(${candidates//:/\\:})
    candidates=(${candidates//$'\t'/:})
    _describe -V 'wt switch' candidates
}
(( $+functions[compdef] )) && compdef _wt wt
