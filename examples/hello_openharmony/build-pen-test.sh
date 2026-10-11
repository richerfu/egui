#!/usr/bin/env bash
# Build the ordinary demo and a separate UITest HAP. No test runner is in the ordinary HAP.
set -euo pipefail
example_dir="$(cd "$(dirname "$0")" && pwd)"
repo_dir="$(cd "$example_dir/../.." && pwd)"
deveco="${DEVECO_HOME:-/Applications/DevEco-Studio.app/Contents}"
export PATH="$deveco/tools/node/bin:$deveco/tools/ohpm/bin:$PATH"
export JAVA_HOME="$deveco/jbr/Contents/Home"
export DEVECO_SDK_HOME="$deveco/sdk"
"$example_dir/build-hap.sh" "$@"
cd "$repo_dir/target/hello-openharmony-hap"
"$deveco/tools/hvigor/bin/hvigorw" --mode module -p product=default -p module=entry@ohosTest \
    -p buildMode=debug assembleHap --no-daemon
printf '\nUITest HAP: %s/entry/build/default/outputs/ohosTest/entry-ohosTest-unsigned.hap\n' "$PWD"
