# JARVIS Zsh Integration
# Loaded automatically by ~/.zshrc

JARVIS_DIR="$HOME/.config/jarvis"

source "$JARVIS_DIR/prompt.zsh"
source "$JARVIS_DIR/aliases.zsh"
source "$JARVIS_DIR/commands.zsh"

# Display welcome banner exactly once per interactive shell session
if [[ -o interactive && -z "$JARVIS_SESSION_STARTED" ]]; then
    JARVIS_SESSION_STARTED=1

    local os_name
    os_name=$(awk -F= '/^NAME=/{gsub(/"/,""); print $2}' /etc/os-release 2>/dev/null)
    os_name=${os_name:-Linux}

    local host_val
    host_val=$(hostname)

    local user_host="${USER}@${host_val}"

    # Compact UPTIME: e.g. "1h 25m" or "50m"
    local uptime_val="unknown"
    if [[ -r /proc/uptime ]]; then
        local up_secs
        read up_secs _ < /proc/uptime
        up_secs=${up_secs%%.*}
        local days=$(( up_secs / 86400 ))
        local hours=$(( (up_secs % 86400) / 3600 ))
        local mins=$(( (up_secs % 3600) / 60 ))
        if (( days > 0 )); then
            uptime_val="${days}d ${hours}h ${mins}m"
        elif (( hours > 0 )); then
            uptime_val="${hours}h ${mins}m"
        else
            uptime_val="${mins}m"
        fi
    fi

    # Colors
    local C=$'\e[36m'   # Cyan
    local G=$'\e[32m'   # Green
    local A=$'\e[33m'   # Amber / Yellow
    local D=$'\e[90m'   # Dim Gray
    local W=$'\e[97m'   # White
    local B=$'\e[1m'    # Bold
    local R=$'\e[0m'    # Reset

    _jpad() { printf '%*s' "$1" ''; }

    local cols=${COLUMNS:-80}
    local left_w=42
    local box_w=29

    # Calculate gap between columns (safely bounded)
    local gap=$(( cols - left_w - box_w - 4 ))
    if (( gap < 2 )); then gap=2; fi
    if (( gap > 12 )); then gap=12; fi

    local mid=$(_jpad $gap)

    # LEFT COLUMN (42 visible chars wide per line)
    local l1="${C}  ██╗ ████╗ █████╗ ██╗ ██╗██╗█████╗${R}       "
    local l2="${C}  ██║██╔═██╗██╔══██╗██║ ██║██║██╔══╝${R}      "
    local l3="${C}  ██║██████║██████╔╝██║ ██║██║█████╗${R}      "
    local l4="${C}╚███║██╔═██║██╔══██╗╚████╔╝██║╚══██║${R}      "
    local l5="  ${W}${B}JARVIS TERMINAL PORTAL${R}                  "
    local l6="  ${D}Native Zsh Environment${R}                  "
    local l7="$(_jpad $left_w)"

    local l8
    if (( cols < 85 )); then
        l8="  ${D}Type ${G}commands${D} to explore modules${R}        "
    else
        l8="  ${D}Type ${G}commands${D} to explore JARVIS modules${R} "
    fi

    # RIGHT COLUMN (29 visible chars wide per line)
    local r1="${C}╭──── [ ${W}${B}SYSTEM ONLINE${R}${C} ] ────╮${R}"

    local ident_len=$(( ${#os_name} + 3 + ${#user_host} ))
    local pad_ident=$(( 25 - ident_len ))
    if (( pad_ident < 0 )); then pad_ident=0; fi
    local r2="${C}│${R} ${A}${os_name}${R} • ${G}${user_host}${R}$(_jpad $pad_ident) ${C}│${R}"

    local r3="${C}│${R}$(_jpad 27)${C}│${R}"

    local pad_os=$(( 16 - ${#os_name} ))
    if (( pad_os < 0 )); then pad_os=0; fi
    local r4="${C}│${R} ${D}OS${R}       ${A}${os_name}${R}$(_jpad $pad_os) ${C}│${R}"

    local pad_sh=$(( 16 - 3 )) # "Zsh" is 3 chars
    if (( pad_sh < 0 )); then pad_sh=0; fi
    local r5="${C}│${R} ${D}SHELL${R}    ${W}Zsh${R}$(_jpad $pad_sh) ${C}│${R}"

    local pad_h=$(( 16 - ${#host_val} ))
    if (( pad_h < 0 )); then pad_h=0; fi
    local r6="${C}│${R} ${D}HOST${R}     ${W}${host_val}${R}$(_jpad $pad_h) ${C}│${R}"

    local pad_u=$(( 16 - ${#uptime_val} ))
    if (( pad_u < 0 )); then pad_u=0; fi
    local r7="${C}│${R} ${D}UPTIME${R}   ${W}${uptime_val}${R}$(_jpad $pad_u) ${C}│${R}"

    local r8="${C}╰───────────────────────────╯${R}"

    # DIVIDER LINE: spans exact content width
    local div_len=$(( left_w + gap + box_w ))
    local div_chars=""
    local i
    for (( i=0; i<div_len; i++ )); do div_chars+="─"; done
    local div="${C}${div_chars}${R}"

    echo ""
    print -P "${l1}${mid}${r1}"
    print -P "${l2}${mid}${r2}"
    print -P "${l3}${mid}${r3}"
    print -P "${l4}${mid}${r4}"
    print -P "${l5}${mid}${r5}"
    print -P "${l6}${mid}${r6}"
    print -P "${l7}${mid}${r7}"
    print -P "${l8}${mid}${r8}"
    print -P "${div}"
    echo ""

    unfunction _jpad 2>/dev/null
fi
