_default:
  @just --choose

# This script in particular will become very useful once we have more languages
# supported
# Lint the entire codebase
lint:
  cargo fmt
  cargo fix --allow-staged

# Build the selected targets
build *targets:
  #!/usr/bin/env python3
  import subprocess
  import sys
  import os

  class Colour:
      PURPLE = '\033[95m'
      CYAN = '\033[96m'
      DARKCYAN = '\033[36m'
      BLUE = '\033[94m'
      GREEN = '\033[92m'
      YELLOW = '\033[93m'
      RED = '\033[91m'
      BOLD = '\033[1m'
      UNDERLINE = '\033[4m'
      END = '\033[0m'


  if '{{targets}}' == '':
      print(Colour.BOLD + Colour.YELLOW + 'No target specified, building for local machine...'
            + Colour.END)
      if os.getenv('BUILD_MODE') == 'RELEASE':
        subprocess.run(['cargo', 'build', '--release'], check=True)
      else:
        subprocess.run(['cargo', 'build'], check=True)
      sys.exit()

  TARGETS = {
      'linux': 'x86_64-unknown-linux-gnu',
      'windows': 'x86_64-pc-windows-gnu',
      'mac': 'x86_64-apple-darwin',
      'rpi': 'armv7-unknown-linux-gnueabihf',
      'rpi-legacy': 'arm-unknown-linux-gnueabihf'
      # More shall be added soon...
  }
  SELECTED_TARGETS = '{{targets}}'.split(' ')
  TARGET_TRIPLES = []

  if 'all' in SELECTED_TARGETS:
      for target in TARGETS:
          TARGET_TRIPLES.append(TARGETS[target])
  else:
      for target in SELECTED_TARGETS:
          if target in TARGETS:
              TARGET_TRIPLES.append(TARGETS[target])

  if SELECTED_TARGETS == []:
      print(Colour.BOLD + 'No valid TARGETS specified!' + Colour.END)
  else:
      print(Colour.BOLD + Colour.BLUE + 'Target(s):', Colour.CYAN
            + ' '.join(TARGET_TRIPLES) + Colour.END)
      for target in TARGET_TRIPLES:
          print(Colour.BOLD + Colour.CYAN + target + Colour.END)
          print(Colour.BOLD + Colour.BLUE + 'Installing target' + Colour.END)
          subprocess.run(['rustup', 'target', 'add', target], stderr=subprocess.DEVNULL, check=True)

          print(Colour.BOLD + Colour.BLUE + 'Compiling target' + Colour.END)
          
          if os.getenv('BUILD_MODE') == 'RELEASE':
            subprocess.run(['cargo', 'build', '--target', target, '--release'], check=True)
          else:
            subprocess.run(['cargo', 'build', '--target', target], check=True)

  # Rustc has a feature-gated option for multiple targets at once (-Zmultitarget) which may be worth looking into

# Print the system info for use in a bug report
@info:
  echo "Please use the following data when preparing a bug report:"
  echo "Architecture: {{arch()}}"
  echo "Operating system: {{os()}} ({{os_family()}})"
  echo "Commit: {{`git rev-parse --short HEAD`}} ({{`git rev-parse HEAD`}})"
  echo "Branch: {{`git rev-parse --abbrev-ref HEAD`}}"
  echo "Cargo: {{`cargo version`}}"
  echo "Rust: {{`rustc --version`}}"

clean-build *targets:
  cargo clean
  just build {{targets}}

# Compile all project documentation
doc:
  cargo doc
  @# When there is an mdbook it should compile that too

# This should probably be used when running CI
# Output the health status of the codebase
health:
  cargo outdated
  cargo deny check
  cargo cache

# Install the KIROS development toolchain
install-toolchain:
  @if ! command -v rustup &> /dev/null; then \
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh; \
    source $HOME/.cargo/env \
  fi
  rustup update

  cargo install cargo-outdated
  cargo install cargo-deny
  cargo install cargo-cache

build-release:
  BUILD_MODE=RELEASE just clean-build all
  just doc
# release: build-release
  
# Add a script to release & publish latest version (with artifacts, tags etc)
# Add a script that runs in CI
# Add a script that runs all tests
# Add a script that runs all benchmarks
# Add a script that lints all code
