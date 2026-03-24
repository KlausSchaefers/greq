# Code Signing Setup for greq

This guide explains how to set up Apple Developer code signing to eliminate macOS verification warnings.

## 🍎 **Apple Developer Setup**

### Prerequisites
- Apple Developer Account ($99/year)
- macOS with Xcode Command Line Tools

### Step 1: Create Developer ID Certificate

1. Visit [Apple Developer Portal](https://developer.apple.com/account/resources/certificates)
2. Click "+" to create new certificate
3. Select "Developer ID Application" (for distribution outside App Store)
4. Follow the wizard to create and download the certificate

### Step 2: Export Certificate

```bash
# Open Keychain Access
# 1. Find your "Developer ID Application" certificate
# 2. Right-click → Export → Choose Personal Information Exchange (.p12)
# 3. Set a strong password
# 4. Save as DeveloperIDApplication.p12
```

### Step 3: Prepare for GitHub Actions

```bash
# Convert certificate to base64
base64 -i DeveloperIDApplication.p12 | pbcopy
# This copies the base64 string to clipboard
```

### Step 4: Find Your Team ID

```bash
# Method 1: From certificate
security find-identity -v -p codesigning
# Look for your Developer ID, the team ID is in parentheses

# Method 2: From Apple Developer Portal
# Visit developer.apple.com → Account → Team ID is shown at top
```

## 🔐 **GitHub Secrets Configuration**

Add these secrets in your GitHub repo → Settings → Secrets and variables → Actions:

### Required Secrets
- **`APPLE_CERTIFICATE`**: Base64 encoded .p12 certificate file
- **`APPLE_CERTIFICATE_PASSWORD`**: Password for the .p12 file  
- **`APPLE_TEAM_ID`**: Your Apple Developer Team ID (e.g., `ABC123DEF4`)
- **`APPLE_DEVELOPER_ID`**: Your Developer ID name (e.g., `Developer ID Application: Your Name (ABC123DEF4)`)

### Optional (for Notarization)
- **`APPLE_ID`**: Your Apple ID email
- **`APPLE_APP_PASSWORD`**: App-specific password for your Apple ID

#### Creating App-Specific Password
1. Go to [appleid.apple.com](https://appleid.apple.com)
2. Sign in → App-Specific Passwords → Generate
3. Use label like "GitHub Actions greq"

## 🚀 **Workflow Integration**

The GitHub Actions workflow (`.github/workflows/release.yml`) already includes:
- **Code Signing**: Automatically signs macOS binaries
- **Notarization**: Optional step for full Gatekeeper approval
- **Verification**: Confirms signature validity

## 🧪 **Local Development Signing**

For local development, you can sign binaries manually:

```bash
# Build release binary
cargo build --release

# Sign the binary
codesign --force --sign "Developer ID Application: Your Name" \
  --options runtime \
  --timestamp \
  target/release/greq

# Verify signature  
codesign --verify --verbose target/release/greq

# Check code signing info
codesign -dv target/release/greq
```

## 🔍 **Verification**

Once set up, your GitHub releases will include signed binaries that:
- ✅ Run immediately on macOS without warnings
- ✅ Show as verified in Keychain Access
- ✅ Pass Gatekeeper security checks

## 🛠️ **Troubleshooting**

### Common Issues

**"No identity found"**
- Ensure certificate is imported in your local Keychain
- Check Team ID matches exactly

**"The specified item could not be found in the keychain"**  
- Verify certificate password is correct
- Ensure certificate hasn't expired

**"Unsigned binary" warnings persist**
- Check if notarization completed successfully
- Allow 1-2 hours for Apple's CDN to propagate approval

### Testing Signed Binaries

```bash
# Download your signed release
curl -L -o greq https://github.com/your-username/greq/releases/latest/download/greq-macos-arm64

# Make executable  
chmod +x greq

# Should run without warnings
./greq --help
```

## 💰 **Cost Considerations**

- **Apple Developer Program**: $99/year
- **Benefits**: Eliminates user friction, professional distribution
- **Alternative**: Document workarounds for unsigned binaries (current approach)

## 🔒 **Security Best Practices**

- Store certificates securely in GitHub encrypted secrets
- Rotate app-specific passwords annually  
- Monitor certificate expiration dates
- Use least-privilege principles for certificates