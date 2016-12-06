#!/bin/bash
set -e

# Brings in: EXAMPLES_DIR, LIB_DIR, CODEGEN_DIR, and CONTRIB_DIR, DOC_DIR
SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
source $SCRIPT_DIR/config.sh

# Add Cargo to PATH.
export PATH=${HOME}/.cargo/bin:${PATH}

# Builds and tests the Cargo project at $1
function build_and_test() {
  local dir=$1
  if [ -z "${dir}" ] || ! [ -d "${dir}" ]; then
    echo "Tried to build and test inside '${dir}', but it is an invalid path."
    exit 1
  fi

  pushd ${dir}
  echo ":: Building '${PWD}'..."
  RUST_BACKTRACE=1 cargo build --all-features

  echo ":: Running unit tests in '${PWD}'..."
  RUST_BACKTRACE=1 cargo test --all-features
  popd
}

# Checks that the versions for Cargo projects $@ all match
function check_versions_match() {
  local last_version=""
  for dir in $@; do
    local cargo_toml="${dir}/Cargo.toml"
    if ! [ -f "${cargo_toml}" ]; then
      echo "Cargo configuration file '${cargo_toml}' does not exist."
      exit 1
    fi

    local version=$(grep version ${cargo_toml} | head -n 1 | cut -d' ' -f3)
    if [ -z "${last_version}" ]; then
      last_version="${version}"
    elif ! [ "${version}" = "${last_version}" ]; then
      echo "Versions differ in '${cargo_toml}'. ${version} != ${last_version}"
      exit 1
    fi
  done
}

# Update dependencies first.
cargo update

build_and_test "${LIB_DIR}"
build_and_test "${CODEGEN_DIR}"
build_and_test "${CONTRIB_DIR}"

check_versions_match "${LIB_DIR}" "${CODEGEN_DIR}" "${CONTRIB_DIR}"

for file in ${EXAMPLES_DIR}/*; do
  if [ -d "${file}" ]; then
    bootstrap_script="${file}/bootstrap.sh"
    if [ -x "${bootstrap_script}" ]; then
      echo ":: Bootstrapping ${file}..."

      # We're just going to leave this commented out for next time...
      # if [ "$(basename $file)" = "todo" ]; then
      #   echo ":: Skipping todo example due to broken Diesel..."
      #   continue
      # fi

      if ! ${bootstrap_script}; then
        echo ":: Running bootstrap script (${bootstrap_script}) failed!"
        echo ":: Skipping ${file}."
        continue
      fi
    fi

    build_and_test "${file}"
  fi
done
