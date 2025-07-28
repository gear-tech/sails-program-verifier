REPO_URL=$REPO_URL
PROJECT_NAME=$PROJECT_NAME
BUILD_IDL=$BUILD_IDL
MANIFEST_PATH=$MANIFEST_PATH
BASE_PATH=$BASE_PATH
MNT_DIR=/mnt/target
ROOT_DIR=$(pwd)
TARGET_DIR="$ROOT_DIR/target"
RELEASE_DIR="$TARGET_DIR/wasm32-gear/release"

echo "Cloning repository $REPO_URL"
git clone --depth 1 $REPO_URL .

if [ $? -ne 0 ]; then
    echo "Error: Failed to clone the repository $REPO_URL" >&2
    exit 1
fi

if [ -n "$BASE_PATH" ]; then
    cd "$BASE_PATH"
fi

args=

if [ -n "$PROJECT_NAME" ]; then
    args="-p $PROJECT_NAME"
elif [ -n "$MANIFEST_PATH" ]; then
    args="--manifest-path $MANIFEST_PATH"
elif [ -f "Cargo.toml" ]; then
    args=""
else
    echo "Error: Cargo.toml not found in the current directory, cannot resolve project" >&2
    exit 1
fi

cargo build --release $args --target-dir "$TARGET_DIR"

if [ $? -ne 0 ]; then
    exit 1
fi

if [ "$BUILD_IDL" = "true" ]; then
    echo "Building the idl"
    cargo-sails sails idl $args --target-dir "$MNT_DIR"
    if [ $? -ne 0 ]; then
        exit 1
    fi
fi

echo "=== $RELEASE_DIR ==="
ls -al "$RELEASE_DIR"
echo "Copying files..."
cp "$RELEASE_DIR"/* "$MNT_DIR"

echo "=== $MNT_DIR ==="
ls -al "$MNT_DIR"
