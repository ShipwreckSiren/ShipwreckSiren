/**
 * VoltSearch WASM Example
 *
 * This example demonstrates how to use the VoltSearch WASM module
 * in a TypeScript/JavaScript environment.
 *
 * Build instructions:
 * 1. Install wasm-pack: cargo install wasm-pack
 * 2. Build the WASM module: wasm-pack build --target web
 * 3. The generated files will be in pkg/
 */

// Import the WASM module (adjust path based on your setup)
import init, { VoltSearch } from './pkg/voltsearch.js';

async function main() {
  // Initialize the WASM module
  await init();

  // Create a new search index
  const index = new VoltSearch();

  // Index some documents
  // Parameters: (id: number, path: string, content: string, content_hash: bigint)
  index.index_document(1, "notes/rust.md", "Rust is a systems programming language", BigInt(12345));
  index.index_document(2, "notes/wasm.md", "WebAssembly is a binary instruction format", BigInt(23456));
  index.index_document(3, "notes/search.md", "Search engines use inverted indices for fast lookups", BigInt(34567));
  index.index_document(4, "notes/rust-wasm.md", "Rust compiles to WebAssembly for web applications", BigInt(45678));

  // Check if a document needs reindexing
  const needsReindex = index.needs_reindex(1, BigInt(12345));
  console.log("Document 1 needs reindex:", needsReindex); // false

  const needsReindexChanged = index.needs_reindex(1, BigInt(99999));
  console.log("Document 1 needs reindex (different hash):", needsReindexChanged); // true

  // Search for documents (AND logic - all terms must match)
  const results1 = index.search("rust", 10);
  console.log("\nSearch results for 'rust':");
  console.log(JSON.parse(results1));
  // Expected: Documents 1 and 4

  const results2 = index.search("rust webassembly", 10);
  console.log("\nSearch results for 'rust webassembly':");
  console.log(JSON.parse(results2));
  // Expected: Document 4 only (contains both terms)

  const results3 = index.search("search engines", 10);
  console.log("\nSearch results for 'search engines':");
  console.log(JSON.parse(results3));
  // Expected: Document 3 only

  // Remove a document
  index.remove_document(1);
  console.log("\nAfter removing document 1:");

  const results4 = index.search("rust", 10);
  console.log("Search results for 'rust':");
  console.log(JSON.parse(results4));
  // Expected: Document 4 only (document 1 was removed)

  // Update a document (same as indexing with same ID)
  index.index_document(2, "notes/wasm.md", "WebAssembly and Rust work great together", BigInt(99999));

  const results5 = index.search("rust", 10);
  console.log("\nSearch results for 'rust' after updating document 2:");
  console.log(JSON.parse(results5));
  // Expected: Documents 2 and 4
}

// Run the example
main().catch(console.error);

/**
 * Example result format:
 * [
 *   {
 *     "doc_id": 4,
 *     "path": "notes/rust-wasm.md",
 *     "score": 2
 *   },
 *   {
 *     "doc_id": 1,
 *     "path": "notes/rust.md",
 *     "score": 1
 *   }
 * ]
 *
 * Results are sorted by score (descending).
 * Score is the sum of term frequencies for all query terms.
 */
