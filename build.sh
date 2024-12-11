REPO_URL=$REPO_URL
PROJECT_NAME=$PROJECT_NAME
BUILD_IDL=$BUILD_IDL

# Clone the repo exit on error
echo "Clonning repository $REPO_URL"

git clone --depth 1 $REPO_URL .

if [ $? -ne 0 ]; then
    echo "Failed to clone the repository $REPO_URL"
    exit 1
fi

MANIFEST_PATH="pumpum."

if [ -z "$PROJECT_NAME" ]; then
    MANIFEST_PATH=$(cargo locate-project | jq -r '.root')
else
    MANIFEST_PATH=$(cargo metadata --format-version 1 --no-deps | jq -r --arg name "$PROJECT_NAME" '.packages[] | select(.name == $name) | .manifest_path')
fi

echo "Manifest path: $MANIFEST_PATH"

echo "Building the project $PROJECT_NAME"
cargo build --manifest-path $MANIFEST_PATH --release

ls -al target/wasm32-unknown-unknown/release/*.wasm

if [ "$BUILD_IDL" = "true" ]; then
    echo "Building the idl"
    cargo-sails sails idl --manifest-path $MANIFEST_PATH
    ls -al target/*.idl
fi

cp target/wasm32-unknown-unknown/release/*.wasm /mnt/build/
cp target/*.idl /mnt/build/

ls -al /mnt/build
