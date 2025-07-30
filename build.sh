REPO_URL=$REPO_URL
PROJECT_NAME=$PROJECT_NAME
BUILD_IDL=$BUILD_IDL
MANIFEST_PATH=$MANIFEST_PATH
BASE_PATH=$BASE_PATH

ROOT_DIR=$(pwd)
TARGET_DIR="$ROOT_DIR/target"
RELEASE_DIR="$TARGET_DIR/wasm32-gear/release"
MNT_DIR=/mnt/target

echo "Root directory: $ROOT_DIR"
echo "Release directory: $RELEASE_DIR"
echo "Target directory: $TARGET_DIR"
echo "Mount directory: $MNT_DIR"

echo "Cloning repository $REPO_URL"
git clone --depth 1 $REPO_URL .

if [ $? -ne 0 ]; then
    echo "Error: Failed to clone the repository $REPO_URL" >&2
    exit 1
fi

if [ -n "$BASE_PATH" ]; then
    cd "$BASE_PATH"
    echo "Changing directory to $(pwd)"
fi

args=

if [ -n "$PROJECT_NAME" ]; then
    echo "Using project name: $PROJECT_NAME"
    args="-p $PROJECT_NAME"
elif [ -n "$MANIFEST_PATH" ]; then
    echo "Using manifest path: $MANIFEST_PATH"
    if [ ! -f "$MANIFEST_PATH" ]; then
        echo "Error: Manifest path $MANIFEST_PATH not found" >&2
        exit 1
    fi
    args="--manifest-path $MANIFEST_PATH"
elif [ -f "Cargo.toml" ]; then
    echo "Using root Cargo.toml"
    args=""
else
    echo "Error: Cargo.toml not found in the current directory, cannot resolve project" >&2
    exit 1
fi

echo "Run cargo build with $args --target-dir $TARGET_DIR"
RUSTFLAGS="--remap-path-prefix=$(pwd)=/app" cargo build --release $args --target-dir "$TARGET_DIR"

if [ $? -ne 0 ]; then
    echo "Error: Failed to build the project"
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
