# Security

## Reporting a vulnerability
Please use GitHub's private vulnerability reporting (Security tab > Report a vulnerability). Do not open a public issue
for something exploitable. There is no bug bounty; reports are handled on a best-effort basis.

## Supported versions
Only the latest release receives fixes.

## How the app handles trust
- **Updates** come only from the GitHub repository the build was compiled for, and Android installs an update only if it
  is signed with the same key as the installed app.
- **Sources written in Rust** run as WebAssembly with a fuel limit, cannot open sockets or files, and cannot reach local
  or private network addresses. They install only from `.sksource` packages (Ed25519 signature plus a publisher
  fingerprint the user confirmed) or from a signed repository index with rollback protection.
- **Extensions** (Kotlin APKs, inherited from Komikku/Mihon) run inside the app process with full access to it. Install
  them only from repositories you trust.
- The app accepts cleartext HTTP and user-installed certificate authorities because some sources require them
  (inherited default from Komikku).

## Credentials in the code
Third-party tracker registrations inherited from Komikku/Mihon (client ids, and the client secrets that Bangumi and
Kitsu publish for native apps) are in the source. They identify the app to those services and cannot be treated as
secret. Signing keys, `signing.properties` and `google-services.json` are git-ignored and never committed.
