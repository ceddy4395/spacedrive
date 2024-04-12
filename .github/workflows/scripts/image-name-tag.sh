 if [ "$GITHUB_EVENT_NAME" == "release" ]; then
    IMAGE_TAG="${GITHUB_REF##*/}"
else
    IMAGE_TAG="$(git rev-parse --short "$GITHUB_SHA")"
fi
IMAGE_TAG="${IMAGE_TAG,,}"
IMAGE_NAME="${GITHUB_REPOSITORY,,}/server"

echo "Building ${IMAGE_NAME}:${IMAGE_TAG}"

echo "tag=${IMAGE_TAG}" >> "$GITHUB_OUTPUT"
echo "name=${IMAGE_NAME}" >> "$GITHUB_OUTPUT"
echo "repo=${GITHUB_REPOSITORY}" >> "$GITHUB_OUTPUT"
echo "repo_ref=${GITHUB_HEAD_REF:-${GITHUB_REF#refs/heads/}}" >> "$GITHUB_OUTPUT"