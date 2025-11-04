# WorkTimer build recipes

# Display all available recipes
default:
    @just --list

# Build the project in release mode
build:
    cargo build --release

# Run tests
test:
    cargo test

# Run clippy linting
lint:
    cargo clippy -- -D warnings

# Check code formatting
fmt-check:
    cargo fmt -- --check

# Format code
fmt:
    cargo fmt

# Create a release: tags and pushes with the given version
# Usage: just release v0.1.0
release version:
    #!/usr/bin/env bash
    set -euo pipefail
    
    # Validate version format (v followed by semver)
    if ! [[ "{{version}}" =~ ^v[0-9]+\.[0-9]+\.[0-9]+(-[a-zA-Z0-9]+)?$ ]]; then
        echo "❌ Invalid version format: {{version}}"
        echo "✓ Expected format: v0.1.0 or v0.1.0-rc1"
        exit 1
    fi
    
    # Check if tag already exists
    if git rev-parse "{{version}}" >/dev/null 2>&1; then
        echo "❌ Tag {{version}} already exists"
        exit 1
    fi
    
    echo "📦 Creating release {{version}}..."
    git tag "{{version}}"
    echo "✓ Tag created"
    
    echo "🚀 Pushing tag to remote..."
    git push origin "{{version}}"
    echo "✓ Release {{version}} pushed!"
    echo ""
    echo "✨ GitHub Actions will now build and publish pre-built binaries"
    echo "📍 Watch progress at: https://github.com/$(git config --get remote.origin.url | sed 's/.*:\(.*\)\.git/\1/')/actions"
