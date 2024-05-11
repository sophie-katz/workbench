#!/bin/sh

# This script takes a lot of inspiration and behavior from Rustup
# (https://github.com/rust-lang/rustup).

set -e

# Constants
# ==================================================================================================

BUILD_FROM_SOURCE_INSTRUCTIONS_URL="https://github.com/sophie-katz/workbench/blob/main/docs/build-from-source.md"
REPOSITORY_OWNER="sophie-katz"
REPOSITORY_NAME="workbench"

if [ -z "$WORKBENCH_DIR" ]; then
    WORKBENCH_DIR="$HOME/.workbench"
fi

# GitHub API

function fetch_github_api() {
    if [ -n "$GITHUB_TOKEN" ]; then
        if ! curl -sSf -L \
            -H "Accept: application/vnd.github+json" \
            -H "Authorization: Bearer $GITHUB_TOKEN" \
            -H "X-GitHub-Api-Version: 2022-11-28" \
            "$1"; then
            log_error "failed to fetch GitHub API with token: $1"
            exit 1
        fi
    else
        if ! curl -sSf -L \
            -H "Accept: application/vnd.github+json" \
            -H "X-GitHub-Api-Version: 2022-11-28" \
            "$1"; then
            log_error "failed to fetch GitHub API: $1"
        fi
    fi
}

# Logging
# ==================================================================================================
IS_COLOR_SUPPORTED=""
SHOW_DEBUG_LOGS="no"

if [ ! -t 1 ] || [ ! -t 2 ]; then
    IS_COLOR_SUPPORTED="no"
else
    case "${TERM}" in
        xterm*|rxvt*|urxvt*|linux*|vt*)
            IS_COLOR_SUPPORTED="yes"
        ;;
        *)
            IS_COLOR_SUPPORTED="no"
        ;;
    esac
fi

function is_color_supported() {
    if [ "$IS_COLOR_SUPPORTED" == "yes" ]; then
        return 0
    else
        return 1
    fi
}

function log_note() {
    if is_color_supported; then
        printf "\033[1;90mnote:\033[0m $*\n" 1>&2
    else
        printf "note: $*\n" 1>&2
    fi
}

function log_debug() {
    if [ "$SHOW_DEBUG_LOGS" == "yes" ]; then
        if is_color_supported; then
            printf "\033[0;35mdebug:\033[0m $*\n" 1>&2
        else
            printf "debug: $*\n" 1>&2
        fi
    fi
}

function log_info() {
    if is_color_supported; then
        printf "\033[1;34minfo:\033[0m $*\n" 1>&2
    else
        printf "info: $*\n" 1>&2
    fi
}

function log_warning() {
    if is_color_supported; then
        printf "\033[0;33mwarning:\033[0m $*\n" 1>&2
    else
        printf "warning: $*\n" 1>&2
    fi
}

function log_error() {
    if is_color_supported; then
        printf "\033[1;31merror:\033[0m $*\n" 1>&2
    else
        printf "error: $*\n" 1>&2
    fi
}

# Command line arguments
# ================================================================================================== 
COMMAND_LINE_ARGUMENT_TARGET_TRIPLE=""
COMMAND_LINE_ARGUMENT_RELEASE=""

function parse_command_line_arguments() {
    LAST_COMMAND_LINE_ARGUMENT=""

    for arg in $*; do
        case "$LAST_COMMAND_LINE_ARGUMENT" in
            -t|--target-triple)
                COMMAND_LINE_ARGUMENT_TARGET_TRIPLE="$arg"
                LAST_COMMAND_LINE_ARGUMENT=""
                continue
            ;;
            -r|--release)
                COMMAND_LINE_ARGUMENT_RELEASE="$arg"
                LAST_COMMAND_LINE_ARGUMENT=""
                continue
            ;;
        esac

        LAST_COMMAND_LINE_ARGUMENT="$arg"

        case "$arg" in
            -h|--help)
                echo "usage: $0 [options]"
                echo
                echo "options:"
                echo "  -h, --help"
                echo "    Display this help."
                echo
                echo "  -V, --version"
                echo "    Display the version of this script."
                echo
                echo "  -v, --verbose"
                echo "    Enable verbose output."
                echo
                echo "  -t, --target-triple <target-triple>"
                echo "    Override the detected target triple."
                echo
                echo "  -r, --release <release>"
                echo "    Install the specified release instead of the latest one."
                exit 0
            ;;
            -V|--version)
                echo "Workbench installer $(get_latest_release)"
                exit 0
            ;;
            -v|--verbose)
                SHOW_DEBUG_LOGS="yes"
            ;;
            -t|--target-triple)
                continue
            ;;
            -r|--release)
                continue
            ;;
            *)
                log_error "unknown argument '$arg'"
                exit 1
            ;;
        esac
    done
}

# Target triple detection
# ==================================================================================================

function get_target_architecture() {
    case $(uname -m) in
        arm64)
            echo "aarch64"
        ;;
        x86_64)
            echo "x86_64"
        ;;
        i686)
            echo "i686"
        ;;
        *)
            log_error "unable to detect architecture by machine name"
            log_note "detected machine name is '$(uname -m)'"
            log_note "supported machine names:\n  - arm64\n  - x86_64\n  - i686"
            log_note "please follow these instructions to build from source: $BUILD_FROM_SOURCE_INSTRUCTIONS_URL"
            exit 1
        ;;
    esac
}

function get_target_vendor_os_standard_library() {
    case $(uname -s) in
        Darwin)
            echo "apple-darwin"
            ;;
        Linux)
            echo "unknown-linux-gnu"
            ;;
        *)
            log_error "unable to detect operating system by kernel name"
            log_note "kernel name is '$(uname -s)'"
            log_note "supported kernel names:\n  - Linux\n  - Darwin"
            log_note "please follow these instructions to build from source: $BUILD_FROM_SOURCE_INSTRUCTIONS_URL"
            exit 1
        ;;
    esac
}

function get_target_triple() {
    echo "$(get_target_architecture)-$(get_target_vendor_os_standard_library)"
}

function resolve_target_triple() {
    if [ -n "$COMMAND_LINE_ARGUMENT_TARGET_TRIPLE" ]; then
        echo "$COMMAND_LINE_ARGUMENT_TARGET_TRIPLE"
    else
        get_target_triple
    fi
}

# Version resolution
# ==================================================================================================

ALL_RELEASES=""

function get_all_releases() {
    if [ -z "$ALL_RELEASES" ]; then
        log_debug "fetching all releases from GitHub API..."

        JSON="$(fetch_github_api https://api.github.com/repos/$REPOSITORY_OWNER/$REPOSITORY_NAME/releases)"

        log_debug "request complete"

        ALL_RELEASES="$(echo $JSON \
            | grep -o -e '"name": "[^"]\+"' \
            | grep -o -e '\d\+\.\d\+\.\d\+')"

        if [ -z "$ALL_RELEASES" ]; then
            log_error "no releases found from GitHub API"
            log_debug "JSON response: $JSON"
            exit 1
        fi
    fi

    echo $ALL_RELEASES
}

function get_latest_release() {
    get_all_releases | cut -d ' ' -f 1
}

function does_release_exist() {
    TAG_NAME="v$1"

    log_debug "fetching release with tag '$TAG_NAME' from GitHub API..."

    if fetch_github_api https://api.github.com/repos/$REPOSITORY_OWNER/$REPOSITORY_NAME/releases/tags/$TAG_NAME >/dev/null 2>/dev/null; then
        return 0
    else
        return 1
    fi
}

function resolve_release() {
    if [ -n "$COMMAND_LINE_ARGUMENT_RELEASE" ]; then
        if does_release_exist "$COMMAND_LINE_ARGUMENT_RELEASE"; then
            echo "$COMMAND_LINE_ARGUMENT_RELEASE"
        else
            log_error "release '$COMMAND_LINE_ARGUMENT_RELEASE' does not exist"
            exit 1
        fi
    else
        get_latest_release
    fi
}

# Binary
# ==================================================================================================

function download_binary_artifact() {
    TARGET_TRIPLE="$1"
    RELEASE="$2"

    log_info "downloading binary for release '$RELEASE' and target triple '$TARGET_TRIPLE'..."

    ARCHIVE_NAME="workbench-$RELEASE-$TARGET_TRIPLE.tar.gz"
    URL="https://github.com/$REPOSITORY_OWNER/$REPOSITORY_NAME/releases/download/v$RELEASE/$ARCHIVE_NAME"

    mkdir -p $WORKBENCH_DIR

    if ! curl -sSf -L -o $WORKBENCH_DIR/$ARCHIVE_NAME "$URL"; then
        log_error "failed to download binary artifact: $URL"
        exit 1
    fi

    echo $WORKBENCH_DIR/$ARCHIVE_NAME
}

function install_binary_artifact() {
    ARCHIVE_PATH="$1"

    log_info "installing binary to '$WORKBENCH_DIR'..."

    EXTRACTION_DIR="$WORKBENCH_DIR/$(basename $ARCHIVE_PATH .tar.gz)"

    if ! mkdir -p $EXTRACTION_DIR; then
        log_error "unable to create directory '$EXTRACTION_DIR'"
        exit 1
    fi

    if ! tar -xzf $ARCHIVE_PATH -C $EXTRACTION_DIR; then
        log_error "unable to extract '$ARCHIVE_PATH' to '$EXTRACTION_DIR'"
        exit 1
    fi

    if ! cp $EXTRACTION_DIR/LICENSE.txt $WORKBENCH_DIR; then
        log_error "unable to copy license file '$EXTRACTION_DIR/LICENSE.txt' to '$WORKBENCH_DIR'"
        exit 1
    fi

    if ! mkdir -p $WORKBENCH_DIR/bin; then
        log_error "unable to create directory '$WORKBENCH_DIR/bin'"
        exit 1
    fi

    if ! cp $EXTRACTION_DIR/target/*/release/wb $WORKBENCH_DIR/bin; then
        log_error "unable to copy binary '$EXTRACTION_DIR/target/*/release/wb' to '$WORKBENCH_DIR/bin'"
        exit 1
    fi

    if ! chmod +x $WORKBENCH_DIR/bin/wb; then
        log_error "unable to make binary file '$WORKBENCH_DIR/bin/wb' executable"
        exit 1
    fi

    if ! rm -r $EXTRACTION_DIR $ARCHIVE_PATH; then
        log_error "unable to remove temporary files"
        exit 1
    fi

    log_info "installation successful"
}

function print_install_instructions() {
    echo
    echo "You still need to add this to your shell profile (e.g. ~/.bashrc):"
    echo
    echo "export WORKBENCH_DIR=\"$WORKBENCH_DIR\""
    echo "export PATH=\"\$WORKBENCH_DIR/bin:\$PATH\""
    echo
    echo "And then restart your shell. You can also run these commands now to update your current shell."
}

# Main
# ==================================================================================================

parse_command_line_arguments $@

if [ -n "$COMMAND_LINE_ARGUMENT_TARGET_TRIPLE" ]; then
    log_debug "overriding target triple with '$COMMAND_LINE_ARGUMENT_TARGET_TRIPLE'"
fi

if [ -n "$COMMAND_LINE_ARGUMENT_RELEASE" ]; then
    log_debug "overriding release to install with '$COMMAND_LINE_ARGUMENT_RELEASE'"
fi

ARCHIVE_NAME="$(download_binary_artifact "$(resolve_target_triple)" "$(resolve_release)")"

install_binary_artifact "$ARCHIVE_NAME"

print_install_instructions
