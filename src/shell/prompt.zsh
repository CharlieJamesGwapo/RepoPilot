# Sourced after the user's interactive configuration, in this session only.
_repopilot_prompt() {
  local rp_code=$?
  emulate -L zsh
  local rp_folder=${PWD:t} rp_branch='' rp_git='' rp_status=''
  [[ $PWD == $HOME ]] && rp_folder='~'
  [[ -z $rp_folder ]] && rp_folder='/'
  rp_folder=${rp_folder//[[:cntrl:]]/?}
  rp_folder=${rp_folder//\%/%%}
  if (( $+commands[git] )); then
    rp_branch=$(command git --no-optional-locks -c core.fsmonitor=false symbolic-ref --quiet --short HEAD 2>/dev/null) || {
      rp_branch=$(command git --no-optional-locks -c core.fsmonitor=false rev-parse --short HEAD 2>/dev/null) && rp_branch="@$rp_branch"
    }
    if [[ -n $rp_branch ]]; then
      rp_branch=${rp_branch//[[:cntrl:]]/?}
      rp_branch=${rp_branch//\%/%%}
      rp_git=" git:($rp_branch)"
    fi
  fi
  (( rp_code )) && rp_status=" [exit $rp_code]"
  if [[ -z ${NO_COLOR+x} && ${TERM:-dumb} != dumb ]]; then
    PROMPT="%F{cyan}[RepoPilot]%f %F{blue}${rp_folder}%f%F{magenta}${rp_git}%f%F{red}${rp_status}%f"$'\n''> '
  else
    PROMPT="[RepoPilot] ${rp_folder}${rp_git}${rp_status}"$'\n''> '
  fi
  RPROMPT=''
  # Treat folder and branch text as data, never as shell substitutions.
  unsetopt localoptions
  unsetopt promptsubst promptbang
  setopt promptpercent
  return 0
}
autoload -Uz add-zsh-hook
add-zsh-hook -d precmd _repopilot_prompt 2>/dev/null
add-zsh-hook precmd _repopilot_prompt
