# pass command to rasdf; but remove the history number on the way
export PROMPT_COMMAND='rasdf add $PWD $(history 1 | sed -e '"'"'s/^\s*\(\S\+\s\+\)//'"'"')'

# Case-insensitive and lax matching
export RASDF_FLAGS='il'

z() { 
  local newdir
  newdir=$( rasdf find -dsi $@ )
  if [ -n "$newdir" ]; then 
    cd "$newdir"
  else
    return 2
  fi 
}

alias a='rasdf find -aD '
alias s='rasdf find-all -ail '
alias d='rasdf find -d '
alias f='rasdf find -f '
alias v='nvim $( f $@ )'

rasdf clean
