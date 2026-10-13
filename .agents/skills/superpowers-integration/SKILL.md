---
name: superpowers-integration
description: Prescribes how to invoke superpowers skills (brainstorming, test-driven-development, systematic-debugging, verification-before-completion, executing-plans) across all stages of agi development.
---

# Superpowers Skills Integration Guide for AGI

Every phase of `agi` development must actively leverage the superpowers process skills:

1. **New Feature Implementation**:
   - MUST invoke `test-driven-development` (`superpowers:test-driven-development`) to write failing unit tests before implementing core algorithms.
2. **Bug Fixing & Diagnostic**:
   - MUST invoke `systematic-debugging` (`superpowers:systematic-debugging`) to trace root causes before touching code.
3. **Execution of Roadmaps**:
   - MUST invoke `executing-plans` (`superpowers:executing-plans`) to track task-by-task execution.
4. **Before Claiming Completion**:
   - MUST invoke `verification-before-completion` (`superpowers:verification-before-completion`). Run full commands (`cargo test --workspace`, etc.) and show output evidence before asserting success.
5. **Pre-Commit Enforcement**:
   - Never commit without fresh verification pass. The commit-gate hook will strictly enforce workspace compilability and test passage.
