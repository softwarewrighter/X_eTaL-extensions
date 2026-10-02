# X_eTaL-extensions tasks. Recipes call scripts/*.sh, which hold the
# logic and work without just too. `just` alone lists the recipes.

set positional-arguments

# List the recipes
default:
    @just --list

# The full pre-commit gate
gate:
    scripts/gate.sh

# Show the agentrail saga state and the current step
status:
    agentrail status

# Open the saga plan
plan:
    @cat docs/plan.md
