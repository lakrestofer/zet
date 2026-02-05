# This File Tests Task Lists and Checkboxes

This document contains various task list formats to test checkbox parsing functionality.

## Unchecked Tasks Section

This section contains tasks that are not yet completed:

- [ ] This is an unchecked task item
- [ ] This is another unchecked task
- [ ] This unchecked task has multiple words in its description
- [ ] This task has a [[link-to-page]] within it
- [ ] This task has an [external link](https://example.com) in it

## Checked Tasks Section

This section contains completed tasks:

- [x] This is a checked/completed task
- [x] This is another completed task
- [x] This completed task references [[another-page]]
- [x] This done task has been finished
- [x] Final completed task in this section

## Mixed Task Lists

This section mixes checked and unchecked tasks:

- [ ] Unchecked task one
- [x] Checked task one
- [ ] Unchecked task two
- [x] Checked task two
- [ ] Unchecked task three

## Nested Task Lists

Tasks can be nested within other list items:

- Regular list item
  - [ ] Nested unchecked task
  - [x] Nested checked task
- Another regular item
  - [ ] Another nested task
    - [ ] Double-nested unchecked task
    - [x] Double-nested checked task

## Tasks with Rich Content

- [ ] This task has **bold text** in it
- [x] This task has *italic text* in it
- [ ] This task has `inline code` in it
- [x] This task has a [[wiki-link]] and [markdown link](https://test.com)

## Project Task List Example

- [ ] Plan the project structure
- [ ] Set up development environment
- [x] Initialize git repository
- [x] Create README file
- [ ] Implement core features
  - [x] Feature A completed
  - [ ] Feature B in progress
  - [ ] Feature C not started
- [ ] Write tests
- [ ] Deploy to production
