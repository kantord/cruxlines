# cruxlines

cruxlines analyzes a codebase and ranks symbol definitions by how often they are
referenced, with a bias toward references coming from "hot" files. It outputs a
tab-separated list that can be used to find the most central symbols in a repo.

## What it does

- Parses source files with tree-sitter.
- Finds definitions and references across files.
- Builds a file-level reference graph and computes a file rank.
- Weights references by file rank and git frecency.
- Outputs one line per definition, with optional reference locations.

## How it works

cruxlines works in two layers:

1) File graph
   - A file-level graph is built from usage file -> definition file edges.
   - PageRank is computed on this small graph to get a per-file rank.

2) Definition scoring
   - Each definition gets a local score based on how many references it has.
   - References are weighted by the rank of the file they come from.
   - If a name is defined multiple times, the score is divided by the number
     of definitions to reduce name-collision noise.
   - Final score = local_score * file_rank(definition_file).

The output includes all components so you can interpret the score.

## Heuristics (and why)

The goal is to keep logic simple and avoid heavy per-language semantics:

- Python: only top-level definitions/assignments (importable symbols).
- JavaScript/TypeScript: only exported declarations (importable symbols).
- Rust: only top-level items (importable symbols).
- References are name-based, which is fast and language-agnostic.
- Name collisions are smoothed by splitting score across same-name definitions.

These heuristics are not semantically perfect, but they keep complexity low
while producing useful rankings.

## CLI usage

Analyze the current repo:

```
cruxlines .
```

Show reference locations:

```
cruxlines -u .
```

## Output format

Each line is tab-separated:

```
score    local_score    file_rank    symbol    def_path:line:col    [ref_path:line:col...]
```

`-u/--references` controls whether reference locations are printed.

## Supported languages

- Python (`.py`)
- JavaScript (`.js`, `.jsx`)
- TypeScript (`.ts`, `.tsx`)
- Rust (`.rs`)

## Git ignore behavior

- Directory scans respect gitignore and common ignore files.
- Explicit file arguments are always processed (like ripgrep).

## Notes

cruxlines uses git history to compute frecency for files via the `frecenfile`
crate. If no git repository is found, frecency defaults to neutral weighting.
