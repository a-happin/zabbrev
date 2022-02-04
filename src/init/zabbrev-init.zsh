zle -N __zabbrev::expand
zle -N __zabbrev::expand-and-insert-self
zle -N __zabbrev::expand-and-accept-line
zle -N __zabbrev::insert-space

__zabbrev::expand() {
    local out exit_code
    out="$(zabbrev expand --lbuffer="$LBUFFER" --rbuffer="$RBUFFER")"
    exit_code="$?"
    [ "$exit_code" -eq 0 ] && eval "$out"
}

__zabbrev::expand-and-insert-self() {
    zle __zabbrev::expand
    [[ $__zabbrev_redraw -eq 1 ]] && zle reset-prompt
    [[ $__zabbrev_no_space -ne 1 ]] && zle self-insert
    unset __zabbrev_no_space __zabbrev_redraw
}

__zabbrev::expand-and-accept-line() {
    zle __zabbrev::expand
    zle reset-prompt
    zle accept-line
}

__zabbrev::insert-space() {
    LBUFFER+=" "
}
