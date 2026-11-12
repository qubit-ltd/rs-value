# Code coverage reports

This file is a navigation and evidence guide. It does not record a coverage snapshot for the current source tree. The hosted report can change when a newer workflow run publishes its artifacts, so cite a specific run and source revision when reporting coverage.

- [CI workflow runs](https://github.com/qubit-ltd/rs-value/actions/workflows/ci.yml)
- [Published coverage report](https://qubit-ltd.github.io/rs-value/coverage/)
- Generate a local report with `./.infra/bin/coverage.sh`.

For a reproducible coverage result, record:

- source commit and whether the worktree was dirty; include the relevant diff or patch when it was dirty;
- exact command and enabled Cargo features;
- coverage tool and toolchain versions;
- excluded files or code, if any;
- CI run URL, or the paths to local report artifacts such as `coverage.txt`, `lcov.info`, and `target/llvm-cov/html/index.html`.

Do not infer a current baseline from percentages without that provenance.
