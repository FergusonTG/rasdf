# pass command to rasdf; but remove the history number on the way
export PROMPT_COMMAND='rasdf add $PWD $(history 1 | sed -e '"'"'s/^\s*\(\S\+\s\+\)//'"'"')'

# Case-insensitive but strict matching
export RASDF_FLAGS='is'

z() { 
  local newdir
  newdir=$( rasdf find -dsi $@ )
  if [ -n "$newdir" ]; then 
    printf "%s\n" "$newdir" >&2
    cd "$newdir"
  else
    printf "Can't find matching directory\n" >&2
    return 2
  fi 
}

alias a='rasdf find -Dacl '
alias s='rasdf find-all -ail '
alias d='rasdf find -dsi '
alias f='rasdf find -fsi '
function v {
  filename="$( rasdf find -fsi $* )"
  if [ -f "$filename" ] 
    then nvim "$filename"
    else echo cannot find "$filename" >&2
  fi
}

rasdf clean
