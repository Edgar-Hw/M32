# M32 Game File Policy

Status: LOCKED BASELINE
Task: `0.0.1-T005`

## 1. Purpose

M32 is an emulator, preservation, analysis, and personal-library application.

M32 does not operate as a commercial-game download service, ROM/game package
repository, marketplace, or file-sharing service.

## 2. User-provided game files

M32 is designed around files that the user chooses to import from their own
local storage.

Users are responsible for determining whether they have the right to possess,
copy, import, back up, analyze, or use a particular game file in their
jurisdiction and circumstances.

M32 does not make a legal-ownership determination from a filename, hash,
metadata field, or successful import.

## 3. Files M32 must not bundle or distribute by default

The official M32 application, repository, installer, portable package, test
fixtures, examples, documentation bundle, and update artifacts must not
contain unauthorized commercial game packages or assets.

This includes, unless M32 has explicit redistribution permission:

- commercial JAR/JAD game packages;
- WIPI, SKVM, Clet, LGT or other carrier/manufacturer game binaries;
- ROM-like dumps or extracted executable payloads;
- commercial game graphics, music, fonts, text, videos, or other copyrighted
  assets copied into M32 sample/test data;
- user-imported game files or save data.

Synthetic fixtures, self-authored demonstration programs, and third-party
examples with verified redistribution permission may be included when their
license/source/hash are recorded in the fixture or third-party manifest.

## 4. No game-download feature in v1.0

M32 v1.0 must not:

- search the web for commercial game files on behalf of the user;
- provide links whose purpose is to obtain unauthorized commercial game
  packages;
- host or mirror commercial game files;
- provide an in-app commercial-game store or download catalog;
- automatically upload an imported game file to an M32 service.

The empty-library UI should instruct the user to add a game file they already
possess. It must not steer the user toward piracy-oriented download sources.

## 5. Hashes and metadata

M32 may calculate local cryptographic hashes such as SHA-256 to:

- identify duplicate files;
- attach local compatibility settings;
- correlate a user-owned file with a compatibility profile;
- verify backups or preservation metadata.

A hash is an identifier, not proof of ownership.

Future community compatibility systems may exchange a game hash and settings
only under the privacy/opt-in rules of their corresponding specification.
They must not exchange the game binary itself.

## 6. Local analysis and resource viewing

Game DNA and Game Archaeology may inspect user-imported files locally.

Local analysis does not grant M32 permission to redistribute extracted
graphics, audio, text, fonts, code, or other game resources.

Resource-export features must remain user-directed and local. Official M32
services and release packages must not automatically republish extracted
commercial assets.

## 7. Saves, backups, and diagnostic bundles

M32 save/backup features must not silently include the original game package.

Diagnostic bundles must exclude game binaries by default.

A feature that exports user save data or locally extracted resources must make
the selected contents clear to the user before export.

## 8. Test-fixture policy

Repository tests must prefer:

1. synthetic fixtures authored for M32;
2. upstream fixtures whose redistribution license has been verified;
3. tiny malformed/generated files created specifically for parser/security
   tests.

A commercial game must not be committed merely because it is useful for
testing.

Local developer-only compatibility testing with a personally provided game
file must keep that file outside Git tracking and CI artifacts.

## 9. Removal and incident handling

If an unauthorized game binary or asset is accidentally committed or included
in a release artifact:

1. stop further distribution of the affected artifact;
2. remove the file from the active tree/artifact;
3. assess whether Git history or published release assets also require
   removal;
4. replace the fixture with a synthetic or properly licensed equivalent;
5. record the incident and prevention change in the project worklog;
6. rerun repository/release content checks before publishing again.

## 10. Change control

Changing this policy to permit M32-hosted commercial-game distribution,
automatic game-file upload, or a game-download catalog is a product-scope
change and requires an M32 RFC plus the appropriate MASTER SPEC version
change.

This document is an engineering/product policy and is not legal advice.
