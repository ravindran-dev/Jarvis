autoload -U colors && colors
setopt PROMPT_SUBST

jarvis_git_prompt() {
    local branch
    branch=$(git symbolic-ref --short HEAD 2>/dev/null)
    if [[ -n "$branch" ]]; then
        printf " %%F{cyan}─%%f %%F{magenta}%s%%f" "$branch"
    fi
}

jarvis_precmd() {
    local exit_code=$?

    local os_name
    os_name=$(awk -F= '/^NAME=/{gsub(/"/,""); print $2}' /etc/os-release 2>/dev/null)
    os_name=${os_name:-Linux}

    local time_str="%D{%H:%M}"
    local git_str
    git_str="$(jarvis_git_prompt)"

    # Colors
    local c_cyan="%F{cyan}"
    local c_amber="%F{yellow}"
    local c_green="%F{green}"
    local c_red="%F{red}"
    local c_white="%F{white}"
    local c_reset="%f"

    local p_status="${c_green}❯${c_reset}"
    if [[ $exit_code -ne 0 ]]; then
        p_status="${c_red}❯${c_reset}"
    fi

    # Line 1: ╭─ JARVIS ─ Ubuntu ─ ravi@ravi ─ 23:26 ─ ~/Jarvis ─ main
    local p_start="${c_cyan}╭─ JARVIS ─ ${c_amber}${os_name}${c_cyan} ─ ${c_green}%n@%m${c_cyan} ─ ${c_cyan}${time_str}${c_cyan} ─ ${c_white}%~${c_reset}${git_str}"

    # Line 2: ╰─❯
    PROMPT="${p_start}
${c_cyan}╰─${c_reset}${p_status} "
}

# Safely add the hook
autoload -Uz add-zsh-hook
add-zsh-hook precmd jarvis_precmd 2>/dev/null || precmd_functions+=(jarvis_precmd)
