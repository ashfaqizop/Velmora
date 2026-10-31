# Security policy

## Supported versions

Only the latest release gets security fixes. Velmora updates itself on launch, so most people are on it within a day of a release.

## Reporting a vulnerability

Please don't open a public issue for a security problem. Report it privately through GitHub instead: go to the repository's **Security** tab and choose **Report a vulnerability**. If that isn't available to you, email **ashfaqizop@gmail.com** with "Velmora security" in the subject.

Include what you found, how to reproduce it, and what an attacker could do with it. A proof of concept helps but isn't required.

You'll get a reply within a few days. Once a fix ships, the report can be made public, and you'll be credited unless you'd rather not be.

## What's in scope

- The Velmora app: the Tauri commands, the loopback stream server, the update and first-launch download checks, and how sign-in tokens are stored.
- The UniverseAPI engine: anything a hostile site could use to make it read or write local files, run code, or reach beyond the request it was asked to make.

Problems on the third-party sites Velmora reads (broken links, ads, wrong content) are outside this policy; open a normal issue for those.
