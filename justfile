# This script in particular will become very useful once we have more languages
# supported
# Lint the entire codebase
lint:
    cargo fmt
    cargo fix --allow-dirty --allow-staged

# There should be a recipe for getting system info too
