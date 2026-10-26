# 0001. Record architecture decisions

- Status: accepted
- Date: 2026-10-25

## Context

Handset is built in small steps and several of its choices (what to embed, how to render, what to
leave out) are not obvious from the code. Without a record, the reasons get lost.

## Decision

We will keep short architecture decision records in `docs/adr`, using the template in
`0000-template.md`, and add one in the same change as any decision that is hard to reverse.

## Alternatives considered

- Explain decisions only in pull request descriptions: they are hard to find later.
- A single design document: it goes stale and mixes old and current thinking.

## Consequences

Decisions are easy to find and review. Contributors need to write a few paragraphs when they change
direction.
