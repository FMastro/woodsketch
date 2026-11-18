# Signing and notarizing the macOS builds

Without a signature Gatekeeper refuses a macOS build on any machine but the one that built it:
the app is quarantined on download and opening it offers only "move to Bin". The Windows build
is signed in CI by Azure Artifact Signing over OIDC, with no key stored anywhere; macOS has no
equivalent, because Apple will only issue a Developer ID certificate to a paid Apple Developer
Program account and the private key has to exist on a Mac first.

This is the procedure for creating that identity. It needs doing once, and again when the
certificate expires -- read the date rather than assuming a term. Most of it is command line and
can be handed to an agent on a Mac; the two website steps need the account holder, because Apple
requires an Apple ID with two-factor authentication.

Nothing here is secret. The certificate's own details (the identity string, the Team ID) are
public by design -- they are written into every signed binary -- and the private key, the `.p12`
and its password never leave the machine that makes them or the repository secrets they are
pasted into.

## What the release workflow needs

| Secret | What it is |
|---|---|
| `MACOS_CERT_P12_BASE64` | the private key + the Developer ID Application certificate + Apple's G2 intermediate, base64 of one `.p12` |
| `MACOS_CERT_PASSWORD` | the `.p12` export password |
| `MACOS_SIGNING_IDENTITY` | `Developer ID Application: <name> (<team id>)`, read out of the certificate |
| `MACOS_TEAM_ID` | the ten characters in the brackets |
| `MACOS_APPLE_ID` | the Apple ID that owns the membership, for `notarytool` |
| `MACOS_APP_PASSWORD` | an app-specific password for that Apple ID (`abcd-efgh-ijkl-mnop`) |

Signing alone isn't enough: Apple must also **notarize** each build, which is why the last two
exist. A signed but un-notarized app still trips Gatekeeper.

Three more sign the `.pkg` installer, which needs a second certificate of its own (step 7):
`MACOS_INSTALLER_P12_BASE64`, `MACOS_INSTALLER_PASSWORD` and `MACOS_INSTALLER_IDENTITY`.
Without them the app is signed and notarized but the installer around it isn't, so Gatekeeper
warns about the file people double-click and not about what it installs.

## Ground rules

- The private key is the publishing identity. Anything signed with it is signed as the account
  holder, so it belongs in a password manager or an encrypted volume, never in the repository.
- Never print the `.p12`, its base64, or the password into a terminal transcript. Write them to
  files and let `gh secret set` read from the file (below), so the value is never echoed.
- Apple limits how many Developer ID certificates an account may hold. Check the expiry `openssl x509 -enddate` prints rather than assuming
  five years: the one issued in November 2026 runs to 18 December 2027.
  **Don't churn them.** If a step fails in a way this file doesn't cover, stop rather than
  generating a second certificate.

## 0. What's already there

```bash
security find-identity -v -p codesigning          # an existing Developer ID identity?
xcode-select -p                                   # command line tools present?
xcrun notarytool --help >/dev/null && echo "notarytool ok"
```

If a `Developer ID Application` identity is already listed, **stop**: export that one rather
than burning a certificate slot on a new one.

## 1. The keypair and the signing request

A Certificate Signing Request is the public half of a new keypair plus a name, for Apple to
sign. The private half never leaves the Mac.

```bash
mkdir -p ~/woodsketch-signing && chmod 700 ~/woodsketch-signing && cd ~/woodsketch-signing
openssl genrsa -out developerID.key 2048
chmod 600 developerID.key
openssl req -new -sha256 -key developerID.key \
  -out developerID.certSigningRequest \
  -subj "/emailAddress=<apple id email>/CN=<name>/C=US"
openssl req -in developerID.certSigningRequest -noout -verify -subject
```

Apple largely ignores that subject and issues the certificate under the enrolled account name,
but it has to be there, and the request has to be SHA-256 over a 2048-bit RSA key.

**Account holder:** upload `developerID.certSigningRequest` at developer.apple.com →
Certificates, IDs & Profiles → Certificates → **+**, choosing **Developer ID Application** and
the **G2 Sub-CA** profile type. Not "Developer ID Installer" -- that signs `.pkg` installers,
which WoodSketch doesn't ship. Apple returns a `.cer` (usually
`developerID_application.cer`); put it in `~/woodsketch-signing`.

## 2. Apple's intermediate certificate

The `.p12` should carry the chain, or CI ends up with a certificate it can't build a trust path
for.

```bash
curl -fsSLO https://www.apple.com/certificateauthority/DeveloperIDG2CA.cer
openssl x509 -inform DER -in DeveloperIDG2CA.cer -out g2.pem
openssl x509 -in g2.pem -noout -subject      # expect: Developer ID Certification Authority
```

If that URL moves, the file is linked from <https://www.apple.com/certificateauthority/> as
"Developer ID Certification Authority (G2)". Take it from there, not from a mirror.

## 3. One file for CI

```bash
openssl x509 -inform DER -in developerID_application.cer -out cert.pem
openssl x509 -in cert.pem -noout -subject -enddate      # the identity, and when it expires
LC_ALL=C tr -dc 'A-Za-z0-9' </dev/urandom | head -c 32 > p12.password
chmod 600 p12.password
openssl pkcs12 -export -out woodsketch-developer-id.p12 \
  -inkey developerID.key -in cert.pem -certfile g2.pem \
  -passout file:p12.password
chmod 600 woodsketch-developer-id.p12
```

That `subject` line is where the identity string comes from, rather than from memory:

```
subject=UID=ABCDE12345, CN=Developer ID Application: A Name (ABCDE12345), OU=ABCDE12345, ...
```

`CN=` is `MACOS_SIGNING_IDENTITY`, exactly as printed, spacing included. The ten-character
`OU`/`UID` is `MACOS_TEAM_ID`.

## 4. Prove it signs

An identity that imports cleanly but can't sign is a release failure discovered days later, so
sign something throwaway in a throwaway keychain.

```bash
kc="$PWD/verify.keychain-db"
security create-keychain -p temp "$kc"
security unlock-keychain -p temp "$kc"
security import woodsketch-developer-id.p12 -k "$kc" -P "$(cat p12.password)" -T /usr/bin/codesign
orig=$(security list-keychains -d user | tr -d '"' | xargs)
security list-keychains -d user -s "$kc" $orig          # on the search list, or nothing is valid
security set-key-partition-list -S apple-tool:,apple:,codesign: -s -k temp "$kc" >/dev/null
security find-identity -v -p codesigning "$kc"          # one Developer ID Application line

printf 'int main(void){return 0;}' > t.c && cc t.c -o t
codesign --force --options runtime --timestamp \
  --keychain "$kc" --sign "Developer ID Application: A Name (ABCDE12345)" t
codesign -dv --verbose=4 t 2>&1 | grep -E "Authority|TeamIdentifier|Timestamp"
spctl -a -vvv -t exec t          # "rejected" here is expected: nothing has notarized it yet
security list-keychains -d user -s $orig
security delete-keychain "$kc"
rm -f t t.c
```

A keychain that isn't on the user's search list imports fine but shows `0 valid identities`
and `codesign` answers "no identity found": macOS can't build the trust path to Apple's root
from a keychain it isn't searching. CI's temporary keychain needs the same `list-keychains -s`.

`codesign` succeeding with a timestamp and an `Authority=Developer ID Certification Authority`
chain is the proof. `spctl` rejecting an un-notarized binary is normal and not a problem.

## 5. The notarization credential

For a one-person account an app-specific password is the simplest thing that works:
appleid.apple.com → Sign-In and Security → App-Specific Passwords → **+**, named
`woodsketch-notarytool`.

```bash
xcrun notarytool store-credentials woodsketch-ci \
  --apple-id "<apple id email>" --team-id "<team id>" --password "<app specific password>"
xcrun notarytool history --keychain-profile woodsketch-ci | head
```

A history listing, even an empty one, means the credential works. When Apple rejects it the
usual cause is the wrong Team ID rather than the wrong password.

An App Store Connect API key (`.p8` plus Key ID and Issuer ID) also works and rotates more
cleanly, but the `.p8` downloads exactly once, so it's easier to lose. Worth switching to only
if the app-specific password gives trouble.

## 6. Into the repository's secrets

Reading from a file keeps the values out of the terminal and out of shell history:

```bash
R=FMastro/woodsketch
base64 -i woodsketch-developer-id.p12 > p12.base64
gh secret set MACOS_CERT_P12_BASE64 --repo $R < p12.base64
gh secret set MACOS_CERT_PASSWORD   --repo $R < p12.password
printf '%s' 'Developer ID Application: A Name (ABCDE12345)' | gh secret set MACOS_SIGNING_IDENTITY --repo $R
printf '%s' 'ABCDE12345'          | gh secret set MACOS_TEAM_ID      --repo $R
printf '%s' '<apple id email>'    | gh secret set MACOS_APPLE_ID     --repo $R
printf '%s' '<app password>'      | gh secret set MACOS_APP_PASSWORD --repo $R
gh secret list --repo $R
shred -u p12.base64 2>/dev/null || rm -P p12.base64
```

Without `gh` authenticated with admin rights on the repository, write the six values to a
file with mode 600, paste them into GitHub → Settings → Secrets and variables → Actions by
hand, and delete the file.

## 7. The installer certificate, for the `.pkg`

`WoodSketch-osx-arm64-Setup.pkg` is the main macOS download, and an installer package is signed
by a **Developer ID Installer** certificate rather than the Application one. It is a different
type, so issuing it doesn't spend an Application slot, and `productbuild` is what uses it.

Everything above again, with its own key: `openssl genrsa -out developerID-installer.key 2048`,
a CSR from it, **Developer ID Installer** chosen on Apple's page this time, the same G2
intermediate in the `.p12`. Then the three secrets the workflow looks for:

```bash
base64 -i woodsketch-developer-id-installer.p12 > inst.base64
gh secret set MACOS_INSTALLER_P12_BASE64 --repo $R < inst.base64
gh secret set MACOS_INSTALLER_PASSWORD   --repo $R < inst.password
printf '%s' 'Developer ID Installer: A Name (ABCDE12345)' | gh secret set MACOS_INSTALLER_IDENTITY --repo $R
shred -u inst.base64 2>/dev/null || rm -P inst.base64
```

Step 4 doesn't apply to it: a `.pkg` isn't signed by `codesign`, and what it was signed with is
read back with `pkgutil --check-signature <file>.pkg` instead.

## 8. Keep

`developerID.key`, `cert.pem`, `woodsketch-developer-id.p12` and `p12.password`, somewhere safe
and offline (and the installer pair beside them). Losing a key means asking Apple for another
certificate, and the count is limited.

## What the workflow does with them

`release.yml`'s macOS jobs make a keychain for the run, import the identity, put the keychain on
the search list, set its partition list so nothing stops to ask for a password, and store the
notarizing credential as the `woodsketch-ci` profile. `vpk pack` then gets `--signAppIdentity`,
`--notaryProfile` and `--keychain`: it signs the bundle with the hardened runtime and a
timestamp, submits it to Apple, waits, staples the answer into the bundle, and packs that, so
what the installer and the updater carry is already notarized. `--signInstallIdentity` is added
when the installer secrets are there, which signs and notarizes the `.pkg` too. The command line
tool is signed the same way and its own zip submitted separately, since a zip can't be stapled.

Without the secrets the macOS builds still run and ship unsigned, the way the Windows job does
without its Azure variables, so a fork without an Apple account can still cut a release.
