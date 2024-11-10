#!/bin/bash

# Parse command line arguments
ALLOW_DIRTY=false
FORCE=false

while [[ $# -gt 0 ]]; do
    case $1 in
        --allow-dirty-tree)
            ALLOW_DIRTY=true
            shift
            ;;
        -f|--force)
            FORCE=true
            shift
            ;;
        *)
            echo "Unknown option: $1"
            echo "Usage: $0 [--allow-dirty-tree] [-f|--force]"
            exit 1
            ;;
    esac
done

REPO_WITH_OWNER="osteele/stitch-sync"

# Check for clean git tree unless --allow-dirty-tree is specified
if [ "$ALLOW_DIRTY" = false ]; then
    if ! git diff-index --quiet HEAD --; then
        echo "Error: Git tree is not clean. Commit changes or use --allow-dirty-tree"
        exit 1
    fi
fi

# Run tests
echo "Running tests..."
if ! cargo test; then
    echo "Tests failed. Aborting version bump."
    exit 1
fi

# Get latest release info from GitHub API
echo "Fetching latest release information..."
RELEASE_INFO=$(curl -s "https://api.github.com/repos/${REPO_WITH_OWNER}/releases/latest")
if [ $? -ne 0 ]; then
    echo "Failed to fetch release information"
    exit 1
fi

# Extract current version tag
CURRENT_VERSION=$(echo "$RELEASE_INFO" | grep -o '"tag_name": "[^"]*' | cut -d'"' -f4)
if [ -z "$CURRENT_VERSION" ]; then
    echo "Failed to determine latest version"
    exit 1
fi

# Remove 'v' prefix if present
CURRENT_VERSION=${CURRENT_VERSION#v}

# Split version into components
IFS='.' read -r MAJOR MINOR PATCH <<< "$CURRENT_VERSION"

# Increment patch version
NEW_PATCH=$((PATCH + 1))
NEW_VERSION="v${MAJOR}.${MINOR}.${NEW_PATCH}"

# Prepare tag command
TAG_CMD="git tag"
if [ "$FORCE" = true ]; then
    TAG_CMD="$TAG_CMD -f"
fi
TAG_CMD="$TAG_CMD -a $NEW_VERSION -m $NEW_VERSION"

# Prepare push command
PUSH_CMD="git push origin $NEW_VERSION"
if [ "$FORCE" = true ]; then
    PUSH_CMD="$PUSH_CMD --force"
fi

# Create and push tag
echo "Creating new version tag: $NEW_VERSION"
if ! eval "$TAG_CMD"; then
    echo "Failed to create tag"
    exit 1
fi

echo "Pushing tag to remote..."
if ! eval "$PUSH_CMD"; then
    echo "Failed to push tag"
    exit 1
fi

echo "Successfully bumped version to $NEW_VERSION"
