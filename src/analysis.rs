use std::collections::HashMap;
use std::path::PathBuf;

use crate::find_references::{find_references, Location, ReferenceEdge};

#[derive(Debug, Clone)]
pub struct OutputRow {
    pub rank: f64,
    pub definition: Location,
    pub references: Vec<Location>,
}

#[derive(Debug, Clone)]
pub struct ProfileStats {
    pub parse_ms: u128,
    pub score_ms: u128,
    pub definitions: usize,
    pub references: usize,
}

pub fn cruxlines<I>(inputs: I) -> Vec<OutputRow>
where
    I: IntoIterator<Item = (PathBuf, String)>,
{
    let edges: Vec<ReferenceEdge> = find_references(inputs).collect();

    let mut grouped: HashMap<Location, Vec<Location>> = HashMap::new();
    for edge in edges {
        grouped
            .entry(edge.definition.clone())
            .or_default()
            .push(edge.usage);
    }

    let mut name_counts: HashMap<String, usize> = HashMap::new();
    for definition in grouped.keys() {
        *name_counts.entry(definition.name.clone()).or_default() += 1;
    }

    let mut output_rows = Vec::with_capacity(grouped.len());
    for (definition, mut references) in grouped {
        references.sort_by(|a, b| {
            let key_a = (&a.path, a.line, a.column, &a.name);
            let key_b = (&b.path, b.line, b.column, &b.name);
            key_a.cmp(&key_b)
        });
        let name_count = name_counts
            .get(&definition.name)
            .copied()
            .unwrap_or(1) as f64;
        let rank = references.len() as f64 / name_count;
        output_rows.push(OutputRow {
            rank,
            definition,
            references,
        });
    }

    output_rows.sort_by(|a, b| {
        b.rank
            .partial_cmp(&a.rank)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    output_rows
}

pub fn cruxlines_profiled<I>(inputs: I) -> (Vec<OutputRow>, ProfileStats)
where
    I: IntoIterator<Item = (PathBuf, String)>,
{
    let start_parse = std::time::Instant::now();
    let edges: Vec<ReferenceEdge> = find_references(inputs).collect();
    let parse_ms = start_parse.elapsed().as_millis();

    let start_score = std::time::Instant::now();
    let mut grouped: HashMap<Location, Vec<Location>> = HashMap::new();
    for edge in edges {
        grouped
            .entry(edge.definition.clone())
            .or_default()
            .push(edge.usage);
    }

    let mut name_counts: HashMap<String, usize> = HashMap::new();
    for definition in grouped.keys() {
        *name_counts.entry(definition.name.clone()).or_default() += 1;
    }

    let mut output_rows = Vec::with_capacity(grouped.len());
    for (definition, mut references) in grouped {
        references.sort_by(|a, b| {
            let key_a = (&a.path, a.line, a.column, &a.name);
            let key_b = (&b.path, b.line, b.column, &b.name);
            key_a.cmp(&key_b)
        });
        let name_count = name_counts
            .get(&definition.name)
            .copied()
            .unwrap_or(1) as f64;
        let rank = references.len() as f64 / name_count;
        output_rows.push(OutputRow {
            rank,
            definition,
            references,
        });
    }

    output_rows.sort_by(|a, b| {
        b.rank
            .partial_cmp(&a.rank)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    let score_ms = start_score.elapsed().as_millis();

    let definitions = output_rows.len();
    let references = output_rows.iter().map(|row| row.references.len()).sum();

    (
        output_rows,
        ProfileStats {
            parse_ms,
            score_ms,
            definitions,
            references,
        },
    )
}
#[cfg(test)]
mod tests {
    use super::cruxlines;
    use std::path::PathBuf;

    #[test]
    fn analyze_paths_produces_rows() {
        let files = vec![
            (
                PathBuf::from("fixtures/python/main.py"),
                std::fs::read_to_string("fixtures/python/main.py").expect("read"),
            ),
            (
                PathBuf::from("fixtures/python/utils.py"),
                std::fs::read_to_string("fixtures/python/utils.py").expect("read"),
            ),
            (
                PathBuf::from("fixtures/python/models.py"),
                std::fs::read_to_string("fixtures/python/models.py").expect("read"),
            ),
        ];
        let rows = cruxlines(files);
        assert!(!rows.is_empty());
        assert!(rows.iter().any(|row| row.definition.name == "add"));
    }

    #[test]
    fn scores_are_normalized_by_definition_count() {
        let inputs = vec![
            (
                PathBuf::from("a.py"),
                "def foo():\n    pass\n".to_string(),
            ),
            (
                PathBuf::from("b.py"),
                "def foo():\n    pass\n".to_string(),
            ),
            (
                PathBuf::from("c.py"),
                "foo()\n".to_string(),
            ),
        ];
        let rows = cruxlines(inputs);
        let foo_scores: Vec<f64> = rows
            .iter()
            .filter(|row| row.definition.name == "foo")
            .map(|row| row.rank)
            .collect();
        assert_eq!(foo_scores.len(), 2);
        for score in foo_scores {
            assert!((score - 0.5).abs() < 1e-6, "score was {score}");
        }
    }
}
