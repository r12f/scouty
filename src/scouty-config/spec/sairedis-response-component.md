# Sairedis Response Component Propagation - High-Level Spec

## Background & Goals

The sairedis parser (`sairedis_parser.rs`) already has stateful context association — `Get` (`g`) saves its OID context so the subsequent `GetResponse` (`G`) can inherit it. However, the **component (SAI_OBJECT_TYPE)** is not propagated. This means `GetResponse` and `QueryResponse` lines show an empty component field, making it impossible to filter or categorize these responses by object type.

### Reference

- Sairedis parser source: `crates/scouty/src/parser/sairedis_parser.rs`

## Problem Statement

- `GetResponse` (`G`) lines have no component — users cannot filter responses by SAI object type
- `QueryResponse` (`Q`) lines have the same issue
- The fix requires only adding component to the existing stateful propagation mechanism

## User Stories

- As a SONiC engineer, I want `GetResponse` lines to show the same SAI_OBJECT_TYPE as their corresponding `Get` request, so I can filter and categorize responses

## Requirements Breakdown

### P0 — Must Have

- [ ] **Propagate component from `g` (Get) to `G` (GetResponse)** (dependency: none)
  - Add `last_get_component: RefCell<Option<String>>` to `SairedisParser`
  - Save component in the `g` handler alongside existing context save
  - Read component in the `G` handler

- [ ] **Propagate component from `q` (Query) to `Q` (QueryResponse)** (dependency: none)
  - Add `last_query_component: RefCell<Option<String>>` to `SairedisParser`
  - Save component in the `q` handler (note: current `q` handler doesn't extract component — may need to parse it from the query detail)
  - Read component in the `Q` handler

## Functional Requirements

The parser's stateful fields expand from 2 to 4:

| Field | Saved by | Used by |
|-------|----------|---------|
| `last_get_context` (existing) | `g` | `G` |
| `last_get_component` (new) | `g` | `G` |
| `last_query_context` (existing) | `q` | `Q` |
| `last_query_component` (new) | `q` | `Q` |

No changes to the log format, parsing logic, or YAML config. Only the stateful propagation is extended.

## Acceptance Criteria

- [ ] `GetResponse` lines have `component_name` populated with the SAI_OBJECT_TYPE from the preceding `Get`
- [ ] `QueryResponse` lines have `component_name` populated when available from the preceding `Query`
- [ ] Existing tests continue to pass
- [ ] New tests verify component propagation for Get/GetResponse and Query/QueryResponse pairs

## Out of Scope

- Propagating component to `NotifySyncdResponse` (`A`) — these don't have a meaningful object type
- Changing the component format or adding new parsed fields beyond component propagation

## Open Questions

None
