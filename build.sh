#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

INSTALL_DIR="${HOME}/.local/bin"
BUILD_MODE="release"
DO_PATH=1
DO_CONFIG=1
FORCE_CONFIG=0
PATH_UPDATED=0
PATH_PROFILE=""

usage() {
  cat <<'EOF'
Usage: ./build.sh [options]

Build and install icon-maker, create default config, and set PATH.

Options:
  --debug                 Build debug binary instead of release
  --no-path               Skip shell profile PATH update
  --no-config             Skip config creation
  --force-config          Overwrite existing config.toml
  --install-dir <path>    Install directory for icon-maker binary
  -h, --help              Show this help
EOF
}

log_step() {
  printf '[build.sh] %s\n' "$1"
}

err() {
  printf '[build.sh] error: %s\n' "$1" >&2
  exit 1
}

resolve_profile() {
  local shell_name
  shell_name="$(basename "${SHELL:-}")"

  case "$shell_name" in
    zsh)
      echo "${HOME}/.zshrc"
      ;;
    bash)
      if [[ -f "${HOME}/.bash_profile" ]]; then
        echo "${HOME}/.bash_profile"
      else
        echo "${HOME}/.bashrc"
      fi
      ;;
    *)
      echo "${HOME}/.profile"
      ;;
  esac
}

append_path_block() {
  local profile="$1"
  local install_dir="$2"
  local begin_marker="# >>> icon-maker PATH >>>"
  local end_marker="# <<< icon-maker PATH <<<"
  local export_line="export PATH=\"${install_dir}:\$PATH\""

  mkdir -p "$(dirname "$profile")"
  touch "$profile"

  if grep -Fq "$begin_marker" "$profile"; then
    log_step "PATH block already present in ${profile}"
    return 0
  fi

  if grep -Fq "$export_line" "$profile"; then
    log_step "Equivalent PATH export already present in ${profile}"
    return 0
  fi

  if [[ ":${PATH}:" == *":${install_dir}:"* ]]; then
    log_step "Current PATH already contains ${install_dir}; skipping profile edit"
    return 0
  fi

  {
    printf '\n%s\n' "$begin_marker"
    printf '%s\n' "$export_line"
    printf '%s\n' "$end_marker"
  } >>"$profile"

  PATH_UPDATED=1
  PATH_PROFILE="$profile"
  log_step "Appended PATH block to ${profile}"
}

while [[ $# -gt 0 ]]; do
  case "$1" in
    --debug)
      BUILD_MODE="debug"
      shift
      ;;
    --no-path)
      DO_PATH=0
      shift
      ;;
    --no-config)
      DO_CONFIG=0
      shift
      ;;
    --force-config)
      FORCE_CONFIG=1
      shift
      ;;
    --install-dir)
      [[ $# -ge 2 ]] || err "--install-dir requires a value"
      INSTALL_DIR="$2"
      shift 2
      ;;
    -h|--help)
      usage
      exit 0
      ;;
    *)
      usage >&2
      err "unknown option: $1"
      ;;
  esac
done

INSTALL_DIR="${INSTALL_DIR/#\~/${HOME}}"

log_step "Building icon-maker (${BUILD_MODE})"
if [[ "$BUILD_MODE" == "release" ]]; then
  cargo build --release
  BIN_SRC="target/release/icon-maker"
else
  cargo build
  BIN_SRC="target/debug/icon-maker"
fi

[[ -f "$BIN_SRC" ]] || err "binary not found at ${BIN_SRC}"

log_step "Installing binary to ${INSTALL_DIR}"
mkdir -p "$INSTALL_DIR"
cp "$BIN_SRC" "${INSTALL_DIR}/icon-maker"
chmod +x "${INSTALL_DIR}/icon-maker"

CONFIG_ROOT="${XDG_CONFIG_HOME:-${HOME}/.config}"
CONFIG_DIR="${CONFIG_ROOT}/icon-maker"
CONFIG_PATH="${CONFIG_DIR}/config.toml"

if [[ "$DO_CONFIG" -eq 1 ]]; then
  if [[ -f "$CONFIG_PATH" && "$FORCE_CONFIG" -ne 1 ]]; then
    log_step "Config already exists at ${CONFIG_PATH}; keeping existing file"
  else
    log_step "Writing config to ${CONFIG_PATH}"
    mkdir -p "$CONFIG_DIR"
    cat >"$CONFIG_PATH" <<'EOF'
openai_key = ""
model = "gpt-image-1.5"
EOF
  fi
else
  log_step "Skipping config creation (--no-config)"
fi

if [[ "$DO_PATH" -eq 1 ]]; then
  PROFILE_PATH="$(resolve_profile)"
  append_path_block "$PROFILE_PATH" "$INSTALL_DIR"
else
  log_step "Skipping PATH update (--no-path)"
fi

log_step "Done"
printf '\n'
printf 'Binary: %s\n' "${INSTALL_DIR}/icon-maker"
printf 'Config: %s\n' "${CONFIG_PATH}"
printf 'Verify: icon-maker --help\n'

if [[ "$DO_PATH" -eq 1 ]]; then
  if [[ "$PATH_UPDATED" -eq 1 ]]; then
    printf 'Reload shell: source %s\n' "$PATH_PROFILE"
  else
    printf 'PATH update: no changes made\n'
  fi
fi
