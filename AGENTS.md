# Documentation

Read README.md for project documentation. Do not read README.ko.md unless the user explicitly requests it. Exclude README.ko.md from broad documentation reads and searches to avoid loading the same content twice.

# Development history

- Read the latest entries in HISTORY.md before project work to understand recent changes and unfinished items.
- For each project-related user request, add one entry under `프롬프트별 작업 기록` in HISTORY.md before the final response. Use its template, write in Korean, and put the newest entry first. Update the same entry while continuing that request; keep distinct follow-up requests separate.
- Record the local date/time with timezone, request summary, actual changes and reasons, affected files, checks and results, and remaining work. If nothing changed, say `변경 없음` and briefly record the outcome. Updating the history itself does not require another entry.
- Summarize prompts; do not copy secrets, personal information, private session links, or raw conversations. Do not invent earlier requests, commit IDs, or successful checks. Distinguish completed, partial, blocked, and unverified work.
- Preserve previous entries. Link to DECISIONS.md for substantial design rationale and VALIDATION.md for detailed verification instead of duplicating those records. Add a commit reference only when it already exists; a history update does not require creating a commit.
