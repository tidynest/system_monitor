# Comprehensive Codebase Audit Prompt

I need you to use several agents and go through the ENTIRE codebase and documentation of this project (system_monitor), analyse it, and identify every way it could and should be improved and fixed. Be vigilant while doing your analysis — look for bugs, architectural issues, documentation inconsistencies, missing error handling, security concerns, and anything that could be working incorrectly or suboptimally.

I don't wish that you change anything in the code, no git commands that would change the online repos, and that you do this job fully autonomously, without my input, or that you need my approval. Make a backup of the project the first thing you do, and make sure that you can complete this task in the way that I have described.

The end result should be a report of all findings and a plan on how to solve all issues, saved to `docs/COMPREHENSIVE_AUDIT_REPORT.md`.

DO NOT START TO MAKE ANY CHANGES AT ALL! For that I wish to be present.

## Analysis Areas (use parallel agents for each)

1. **Core Monitoring Logic** — Trace the monitoring pipeline end-to-end. Are system metrics (CPU, memory, disk, network) collected accurately? Any data loss, timing issues, or silent failures?

2. **Data Collection & Processing** — How is system data gathered? Are there proc/sysfs parsing bugs, platform-specific edge cases, or incorrect calculations (e.g., CPU percentage, memory usage)?

3. **Display/Reporting** — If there's a UI or CLI output, does it render correctly? Are values formatted properly? Does state update in real-time or is it stale?

4. **Build System & Tests** — Does it compile cleanly? Do all tests pass? Are there untested critical paths? Check clippy, feature flags, and CI/CD.

5. **Documentation** — Are docs accurate? Version numbers consistent? Any contradictions between README, CHANGELOG, and actual code?

Good luck!
