# VoltSearch - Step 1

A minimal high-performance search engine for Obsidian, with the core built in Rust and compiled to WebAssembly.

## Step 1 Features

This is the initial minimal implementation with:

- **Simple keyword AND search**: All query terms must match
- **Document indexing**: Add, update, and remove documents
- **Content hash tracking**: Check if documents need reindexing
- **Term frequency scoring**: Simple sum of term frequencies
- **JSON results**: Easy integration with TypeScript/JavaScript

## Architecture

### Data Structures

- `VoltSearch`: Main search index
  - `term_to_id`: HashMap mapping terms to numeric IDs
  - `id_to_term`: Vector mapping IDs back to terms
  - `postings`: HashMap of term ID to posting lists
  - `documents`: HashMap of document metadata

- `Posting`: Document occurrence of a term
  - `doc_id`: Document identifier
  - `term_freq`: Frequency of term in document

- `DocumentMeta`: Document metadata
  - `path`: Document file path
  - `content_hash`: Hash for change detection
  - `term_count`: Total terms in document

### Tokenization

Simple tokenization pipeline:
1. Convert to lowercase
2. Split on whitespace
3. Trim ASCII punctuation
4. Filter empty tokens

### WASM API

- `new()`: Create a new index
- `index_document(id, path, content, content_hash)`: Index or update a document
- `remove_document(id)`: Remove a document from the index
- `needs_reindex(id, content_hash)`: Check if document needs reindexing
- `search(query, limit)`: Search and return JSON results

## Building

```bash
# Install wasm-pack
cargo install wasm-pack

# Build for web
wasm-pack build --target web

# Build for Node.js
wasm-pack build --target nodejs

# Run tests
cargo test
```

## Usage

See `example.ts` for a complete TypeScript example.

```typescript
import init, { VoltSearch } from './pkg/voltsearch.js';

await init();
const index = new VoltSearch();

// Index a document
index.index_document(1, "note.md", "Hello world", BigInt(12345));

// Search
const results = index.search("hello", 10);
console.log(JSON.parse(results));
```

## Future Steps

Step 1 is intentionally minimal. Future enhancements may include:
- BM25 scoring
- Phrase search
- Fuzzy search
- Trigram index
- Field-based search (title/body)
- Incremental indexing
- Serialization/persistence
- Batch operations
