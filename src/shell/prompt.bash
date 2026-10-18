# Sourced after the user's interactive configuration, in this session only.
_repopilot_capture() { _repopilot_status=$?; return "$_repopilot_status"; }
_repopilot_prompt() {
  local rp_code=${_repopilot_status:-0} rp_branch=''
  _repopilot_folder=${PWD##*/}
  [[ $PWD == "$HOME" ]] && _repopilot_folder='~'
  [[ -z $_repopilot_folder ]] && _repopilot_folder='/'
  _repopilot_folder=${_repopilot_folder//[[:cntrl:]]/?}
  _repopilot_git=''
  _repopilot_exit=''
  if command -v git >/dev/null 2>&1; then
    rp_branch=$(command git --no-optional-locks -c core.fsmonitor=false symbolic-ref --quiet --short HEAD 2>/dev/null) || {
      rp_branch=$(command git --no-optional-locks -c core.fsmonitor=false rev-parse --short HEAD 2>/dev/null) && rp_branch="@$rp_branch"
    }
    if [[ -n $rp_branch ]]; then
      rp_branch=${rp_branch//[[:cntrl:]]/?}
      _repopilot_git=" git:($rp_branch)"
    fi
  fi
  (( rp_code )) && _repopilot_exit=" [exit $rp_code]"
  # Parameter values are expanded as data after PS1 decoding; they are never eval'd.
  shopt -s promptvars
  if [[ -z ${NO_COLOR+x} && ${TERM:-dumb} != dumb ]]; then
    PS1='\[\e[1;36m\][RepoPilot]\[\e[0m\] \[\e[34m\]${_repopilot_folder}\[\e[35m\]${_repopilot_git}\[\e[31m\]${_repopilot_exit}\[\e[0m\]\n> '
  else
    PS1='[RepoPilot] ${_repopilot_folder}${_repopilot_git}${_repopilot_exit}\n> '
  fi
}
if [[ $(declare -p PROMPT_COMMAND 2>/dev/null) == 'declare -a '* ]]; then
  PROMPT_COMMAND=(_repopilot_capture "${PROMPT_COMMAND[@]}" _repopilot_prompt)
else
  # Newlines preserve valid hooks ending with a semicolon or shell comment.
  PROMPT_COMMAND=$'_repopilot_capture\n'"${PROMPT_COMMAND:-}"$'\n_repopilot_prompt'
fi
