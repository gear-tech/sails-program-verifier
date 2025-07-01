REPO_URL=$REPO_URL
PROJECT_NAME=$PROJECT_NAME
BUILD_IDL=$BUILD_IDL
MANIFEST_PATH=$MANIFEST_PATH
TARGET_DIR=/mnt/target

echo "Clonning repository $REPO_URL"
git clone --depth 1 $REPO_URL .

if [ $? -ne 0 ]; then
    echo "Error: Failed to clone the repository $REPO_URL" >&2
    exit 1
fi

if [ -z "$MANIFEST_PATH" ]; then
    if [ ! -f "Cargo.toml" ]; then
        echo "Error: Cargo.toml not found in the current directory, cannot resolve project" >&2
        exit 1
    elif [ -z "$PROJECT_NAME" ]; then
        MANIFEST_PATH=$(cargo locate-project | jq -r '.root')
    else
        MANIFEST_PATH=$(cargo metadata --format-version 1 --no-deps | jq -r --arg name "$PROJECT_NAME" '.packages[] | select(.name == $name) | .manifest_path')
    fi
fi

cargo build --target-dir=$TARGET_DIR --manifest-path $MANIFEST_PATH --release

if [ $? -ne 0 ]; then
    exit 1
fi

if [ "$BUILD_IDL" = "true" ]; then
    echo "Building the idl"
    cargo-sails sails idl --manifest-path $MANIFEST_PATH --target-dir $TARGET_DIR
    if [ $? -ne 0 ]; then
        exit 1
    fi
fi

cd $TARGET_DIR
cp wasm32-gear/release/*.wasm .
rm -rf release/ wasm-projects/ wasm32-gear/ .rust* debug/ doc/
