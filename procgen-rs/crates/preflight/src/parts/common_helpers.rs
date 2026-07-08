fn region_id(node_id: &str) -> String {
    format!("region.{}", slugify_label(node_id))
}

fn dedupe_strings(values: Vec<String>) -> Vec<String> {
    values
        .into_iter()
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect()
}
