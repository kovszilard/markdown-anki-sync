# Markdown Anki Sync

A CLI tool that syncs flashcards from Markdown files to [Anki](https://apps.ankiweb.net/) via [AnkiConnect](https://foosoft.net/projects/anki-connect/).

Write your flashcards in Markdown, run the tool, and they appear in Anki. The tool tracks sync state by writing metadata back into your Markdown files as HTML comments, so subsequent runs update existing cards rather than creating duplicates.

## Prerequisites

- [Anki](https://apps.ankiweb.net/) running with the [AnkiConnect](https://foosoft.net/projects/anki-connect/) add-on installed (listens on `localhost:8765`)
- [Rust toolchain](https://rustup.rs/) for building from source

## Installation

```sh
cargo install --path .
```

## Usage

```sh
markdown-anki-sync <markdown-file>
```

## Markdown Format

### Front Matter (optional)

Set default deck and tags for all cards in the file:

```markdown
---
anki_sync:
  deck: My Deck
  tags: [tag1, tag2]
---
```

### Flashcards

Define flashcards using a `## Q:` header. Everything after the header until the next header or metadata comment becomes the answer:

```markdown
## Q: What is Rust?

A systems programming language focusing on safety and performance.

## Q: What is a parser combinator?

A higher-order function that takes parsers as input and returns
a new parser as output, allowing complex parsers to be built
from simple ones.
```

Headers with 1-6 `#` levels are supported (`# Q:` through `###### Q:`).

### Per-Card Metadata

Override defaults or disable sync for individual cards using HTML comments:

```markdown
<!-- anki_deck: "Special Deck", anki_tags: [advanced], anki_sync: true -->

## Q: What is ownership in Rust?

A set of rules governing how Rust manages memory.
```

### After Syncing

The tool writes Anki note IDs back into your file so future runs update existing cards:

```markdown
<!-- anki_id: 1734567890, anki_deck: "My Deck", anki_sync: true -->

## Q: What is Rust?

A systems programming language focusing on safety and performance.
```

## Example Workflow

1. Write flashcards in `notes.md`
2. Open Anki (with AnkiConnect installed)
3. Run `markdown-anki-sync notes.md`
4. Output: `Created: 2, Updated: 0`
5. Edit a card in `notes.md` and run again
6. Output: `Created: 0, Updated: 2`

## Building

```sh
cargo build --release
```

## Running Tests

```sh
cargo test
```
