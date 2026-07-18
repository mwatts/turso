use std::collections::HashMap;

#[derive(Debug, Default)]
pub struct QueryParseCache {
    outcomes: HashMap<String, Result<(), String>>,
    requests: usize,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct QueryParseCacheStats {
    pub requests: usize,
    pub unique_queries: usize,
    pub intersections: usize,
}

impl QueryParseCache {
    pub fn parse(&mut self, query: &str) -> Result<(), String> {
        self.requests += 1;
        let key = normalized_query(query);
        self.outcomes
            .entry(key)
            .or_insert_with(|| {
                turso_graph_cypher::parse(query)
                    .map(|_| ())
                    .map_err(|e| e.to_string())
            })
            .clone()
    }

    pub fn stats(&self) -> QueryParseCacheStats {
        QueryParseCacheStats {
            requests: self.requests,
            unique_queries: self.outcomes.len(),
            intersections: self.requests - self.outcomes.len(),
        }
    }
}

fn normalized_query(query: &str) -> String {
    query
        .trim_end_matches(';')
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn equivalent_whitespace_and_terminal_semicolon_share_parse_work() {
        let mut cache = QueryParseCache::default();
        let _ = cache.parse("UNWIND [1, 2] AS x RETURN x;");
        let _ = cache.parse(" UNWIND  [1, 2] AS x\nRETURN x ");
        assert_eq!(cache.stats().requests, 2);
        assert_eq!(cache.stats().unique_queries, 1);
        assert_eq!(cache.stats().intersections, 1);
    }
}
