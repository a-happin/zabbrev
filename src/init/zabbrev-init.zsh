zle -N __zabbrev::expand
zle -N __zabbrev::expand-and-insert-self
zle -N __zabbrev::expand-and-accept-line
zle -N __zabbrev::insert-space

__zabbrev::expand() {
    unset __zabbrev_redraw
    local out
    out="$(zabbrev expand --lbuffer="$LBUFFER" --rbuffer="$RBUFFER")" || return 0
    eval "$out"
    [[ $__zabbrev_redraw -eq 1 ]] && zle reset-prompt
    unset __zabbrev_redraw
    return 0
}

__zabbrev::expand-and-insert-self() {
    unset __zabbrev_no_space
    zle __zabbrev::expand
    [[ $__zabbrev_no_space -ne 1 ]] && zle self-insert
    unset __zabbrev_no_space
    return 0
}

__zabbrev::expand-and-accept-line() {
    zle __zabbrev::expand
    # zle reset-prompt
    zle accept-line
}

__zabbrev::insert-space() {
    LBUFFER+=" "
}
