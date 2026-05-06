# Research: Stratum Selector-v1

## Objective
Replace the expensive LLM-based node selection in `HierarchicalRetriever` with a dedicated, lightweight Cross-Encoder model.

## Task: Node Selection
- **Input**: 
    - `Query`: User's search query.
    - `ParentContext`: Summary of the parent node.
    - `Choices`: List of child node summaries/contents.
- **Output**: Binary relevance or multi-class selection indices.

## Candidate Architecture: Cross-Encoder (TinyBERT / MobileBERT)
- Combine `[Query, Parent, Choice]` into a single sequence.
- Output a logit for relevance.
- Parallelize across all choices.

## Data Collection
Captured via `selector_feeding.jsonl`.
Format: `SelectorTriplet` { query, parent_context, choices, selected_indices }

## Roadmap
1. **Analyze Data**: Inspect `selector_feeding.jsonl` for pattern diversity.
2. **Prototype**: Implement a simple MLP-based selector in Candle that uses existing embeddings.
3. **Training**: Define loss function (Binary Cross Entropy) for relevance.
4. **Integration**: Swap LLM call with `SelectorV1::predict`.
