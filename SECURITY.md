# Security policy

RepoPilot 0.2.x is the currently supported release line. Security fixes target the current release; upgrade older installations before checking whether a problem remains. There is no guaranteed response time or commercial security service.

Report vulnerabilities privately through [GitHub private vulnerability reporting](https://github.com/CharlieJamesGwapo/RepoPilot/security/advisories/new). Do not include working credentials, sensitive source, or exploit details in public issues. If the private form is unavailable, avoid posting sensitive details publicly and check this policy for an updated reporting route.

Include the affected version, environment, a minimal safe reproduction, expected and actual behavior, impact, and any proposed mitigation. Redact keys and private data. Public issues are appropriate for ordinary bugs that do not expose a vulnerability or sensitive information.

See [the security model](docs/security.md) for inspection, native shell execution, optional provider, and skill-export boundaries. Download checksums detect corruption against published values; release binaries are unsigned and macOS binaries are not notarized.
