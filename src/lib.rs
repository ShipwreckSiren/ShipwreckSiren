use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use wasm_bindgen::prelude::*;

// Posting represents a document occurrence of a term
#[derive(Debug, Clone, Serialize, Deserialize)]
struct Posting {
    doc_id: u32,
    term_freq: u32,
}

// DocumentMeta stores metadata about indexed documents
#[derive(Debug, Clone, Serialize, Deserialize)]
struct DocumentMeta {
    path: String,
    content_hash: u64,
    term_count: u32,
}

// SearchResult represents a single search result
#[derive(Debug, Serialize, Deserialize)]
struct SearchResult {
    doc_id: u32,
    path: String,
    score: u32,
}

// VoltSearch is the main search index
#[wasm_bindgen]
pub struct VoltSearch {
    term_to_id: HashMap<String, u32>,
    id_to_term: Vec<String>,
    postings: HashMap<u32, Vec<Posting>>,
    documents: HashMap<u32, DocumentMeta>,
    next_term_id: u32,
}

#[wasm_bindgen]
impl VoltSearch {
    /// Create a new VoltSearch index
    #[wasm_bindgen(constructor)]
    pub fn new() -> VoltSearch {
        VoltSearch {
            term_to_id: HashMap::new(),
            id_to_term: Vec::new(),
            postings: HashMap::new(),
            documents: HashMap::new(),
            next_term_id: 0,
        }
    }

    /// Index or update a document
    pub fn index_document(&mut self, id: u32, path: String, content: String, content_hash: u64) {
        // Remove existing document if present
        self.remove_document(id);

        // Tokenize content
        let tokens = tokenize(&content);
        let term_count = tokens.len() as u32;

        // Count term frequencies
        let mut term_freqs: HashMap<String, u32> = HashMap::new();
        for token in tokens {
            *term_freqs.entry(token).or_insert(0) += 1;
        }

        // Add postings for each term
        for (term, freq) in term_freqs {
            let term_id = self.get_or_create_term_id(term);

            let posting = Posting {
                doc_id: id,
                term_freq: freq,
            };

            self.postings
                .entry(term_id)
                .or_insert_with(Vec::new)
                .push(posting);
        }

        // Store document metadata
        self.documents.insert(
            id,
            DocumentMeta {
                path,
                content_hash,
                term_count,
            },
        );
    }

    /// Remove a document from the index
    pub fn remove_document(&mut self, id: u32) {
        // Remove document metadata
        if self.documents.remove(&id).is_none() {
            return; // Document didn't exist
        }

        // Remove postings for this document
        for postings_list in self.postings.values_mut() {
            postings_list.retain(|p| p.doc_id != id);
        }
    }

    /// Check if a document needs reindexing based on content hash
    pub fn needs_reindex(&self, id: u32, content_hash: u64) -> bool {
        match self.documents.get(&id) {
            Some(meta) => meta.content_hash != content_hash,
            None => true, // Document not indexed, needs indexing
        }
    }

    /// Search for documents matching all query terms (AND logic)
    /// Returns JSON array of results
    pub fn search(&self, query: String, limit: u32) -> String {
        // Tokenize query
        let query_terms = tokenize(&query);

        if query_terms.is_empty() {
            return "[]".to_string();
        }

        // Get term IDs for query terms
        let mut term_ids = Vec::new();
        for term in &query_terms {
            match self.term_to_id.get(term) {
                Some(&id) => term_ids.push(id),
                None => {
                    // Term not in index, no results possible
                    return "[]".to_string();
                }
            }
        }

        // Find documents containing ALL query terms
        let mut doc_scores: HashMap<u32, u32> = HashMap::new();

        // Start with documents containing the first term
        if let Some(postings) = self.postings.get(&term_ids[0]) {
            for posting in postings {
                doc_scores.insert(posting.doc_id, posting.term_freq);
            }
        } else {
            return "[]".to_string();
        }

        // Filter to only documents containing all remaining terms (AND logic)
        for &term_id in &term_ids[1..] {
            if let Some(postings) = self.postings.get(&term_id) {
                let mut new_scores: HashMap<u32, u32> = HashMap::new();

                for posting in postings {
                    if let Some(&existing_score) = doc_scores.get(&posting.doc_id) {
                        // Document contains this term too, add to score
                        new_scores.insert(posting.doc_id, existing_score + posting.term_freq);
                    }
                }

                doc_scores = new_scores;
            } else {
                // Term not found, no documents match
                return "[]".to_string();
            }
        }

        // Convert to results and sort by score
        let mut results: Vec<SearchResult> = doc_scores
            .into_iter()
            .filter_map(|(doc_id, score)| {
                self.documents.get(&doc_id).map(|meta| SearchResult {
                    doc_id,
                    path: meta.path.clone(),
                    score,
                })
            })
            .collect();

        results.sort_by(|a, b| b.score.cmp(&a.score));

        // Apply limit
        if limit > 0 && results.len() > limit as usize {
            results.truncate(limit as usize);
        }

        // Serialize to JSON
        serde_json::to_string(&results).unwrap_or_else(|_| "[]".to_string())
    }

    // Internal helper to get or create a term ID
    fn get_or_create_term_id(&mut self, term: String) -> u32 {
        if let Some(&id) = self.term_to_id.get(&term) {
            return id;
        }

        let id = self.next_term_id;
        self.next_term_id += 1;
        self.term_to_id.insert(term.clone(), id);
        self.id_to_term.push(term);
        id
    }
}

/// Tokenize text: lowercase, split on whitespace, trim punctuation, ignore empty
fn tokenize(text: &str) -> Vec<String> {
    text.to_lowercase()
        .split_whitespace()
        .map(|word| {
            word.trim_matches(|c: char| c.is_ascii_punctuation())
        })
        .filter(|word| !word.is_empty())
        .map(|word| word.to_string())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tokenize() {
        let text = "Hello, World! This is a TEST.";
        let tokens = tokenize(text);
        assert_eq!(tokens, vec!["hello", "world", "this", "is", "a", "test"]);
    }

    #[test]
    fn test_basic_indexing() {
        let mut index = VoltSearch::new();
        index.index_document(1, "doc1.md".to_string(), "hello world".to_string(), 123);

        assert!(index.documents.contains_key(&1));
        assert_eq!(index.documents.get(&1).unwrap().path, "doc1.md");
    }

    #[test]
    fn test_needs_reindex() {
        let mut index = VoltSearch::new();
        index.index_document(1, "doc1.md".to_string(), "hello world".to_string(), 123);

        assert!(!index.needs_reindex(1, 123));
        assert!(index.needs_reindex(1, 456));
        assert!(index.needs_reindex(2, 123));
    }

    #[test]
    fn test_search_and() {
        let mut index = VoltSearch::new();
        index.index_document(1, "doc1.md".to_string(), "hello world".to_string(), 1);
        index.index_document(2, "doc2.md".to_string(), "hello rust".to_string(), 2);
        index.index_document(3, "doc3.md".to_string(), "world rust".to_string(), 3);

        let results = index.search("hello world".to_string(), 10);
        assert!(results.contains("doc1.md"));
        assert!(!results.contains("doc2.md"));
        assert!(!results.contains("doc3.md"));
    }

    #[test]
    fn test_remove_document() {
        let mut index = VoltSearch::new();
        index.index_document(1, "doc1.md".to_string(), "hello world".to_string(), 1);
        index.remove_document(1);

        assert!(!index.documents.contains_key(&1));
        let results = index.search("hello".to_string(), 10);
        assert_eq!(results, "[]");
    }
}
