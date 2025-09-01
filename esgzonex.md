# esgzonex

## meetings

- [[esgzonex-painpoint-meeting]]
- [[esgzonex-summer-plan]]

## Time organization

### 07/03

- 13:33 start clock

## TODO

- bugs
  - 3.3.21
    - [ ] 59 - Need concept of super admin.
    - [ ] 60 - Inconcistend logout, takes some amount of time.
    - [ ] 61 - Document flow for documents, how does supabase/bedrock
          handle file names with whitespace?
    - [ ] 71 - sAXLang does not compute anymore?
    - [ ] 72 - input in wrong category?
      - which template
      - which company?
  - 3.4.0
    - 64 - för många underkategorier fälls ut
    - 65 - för många klick för att "avaktivera" kategori när man
      klickat någon annan stans
    - 66 - samma för att öppna chatten
    - 67 - suggestion level x does not show.
  - 3.6.0
  - [ ] order of tool fields is non deterministic?
  - [ ] only some types are calculated on the rating page
  - [ ] when the owner sends otp

- featurse
  - [[esgzonex-f22-indicator-overlay]]
    - [ ] indicator Id?

## features

- [ ] [[esgzonex-f22-indicator-overlay]]
- [[esgzonex-f21-company-view]]
- [ ] [[esgzonex-f17-team-page]]
- [x] [[esgzonex-f20-integrate-rate]]

## topics

- [[architecture-of-esgzonex]]

## to consider

- [ ] cleanup of old documents, images, etc
  - client should not need to handle syncronizaton of deletion itself
- [ ] want to move handling of documents to supabase edge function
- [ ] want to move processing of function inputs to edge function
- [ ] move admin endpoints under a single
      `/admin/other-endpoints-here` path

## Thoughts

- hotjar functionality for usage statistics

## Summer

- plan
  - agentic tool

- summer plan
  - wednesdays will be free

- time plan for summer and autumn
  - semester

## Features

### Merge together migration files

- there are several redundant files
- I want to move towards a declarative handling of migrations
- and supabase has support for it

### Merge migrations

### Fixing integration with bedrock

I need to update the ai document worker such that it uses the new
expected values in the remote esgzonex-development db.

- [x] change connection details to use esgzonex-production knowledge
      base
- [x] update foreign table document_embedding__remote
- [x] update sync_document_embedding procedure, (remote function do
      longer works)

- **existing queue infra for pipeline?**
