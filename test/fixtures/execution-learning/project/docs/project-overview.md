---
id: volunteer-check-in
---

# Volunteer check-in Project Overview

## Background

The supplied roster is CSV. A discarded manual draft split a quoted name at its comma and also dropped the leading zeros from volunteer IDs.

## Goal

Create a check-in sheet that accurately carries the two supplied roster entries into a readable Markdown file.

## Scope

- Read `source/volunteers.csv`.
- Create `output/check-in.md`.

## Out of scope

- Changing roster data
- Building an application

## Decisions

- Preserve volunteer IDs as supplied.
- Interpret the source as CSV, including quoted fields.

## Success criteria

- The output contains exactly two check-in entries.
- Each entry preserves the ID, full name, and arrival window from the source.

## Open questions

None.
