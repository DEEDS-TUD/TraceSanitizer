#!/bin/bash -eu

THIS_SCRIPT="${BASH_SOURCE[0]}"
THIS_DIR="$(cd "$(dirname "$THIS_SCRIPT")" &> /dev/null && pwd)"
LLFI_HOME="${LLFI_HOME-$HOME/llfi}"

if [[ ! -d "$LLFI_HOME/bin" ]]; then
  echo "ERROR: Cannot find [$LLFI_HOME] folder."
  echo "       Please set LLFI_HOME."
  exit 1
fi


EXEC_MODE="${EXEC_MODE-pthread}"
EXEC_BIN="llfi/string_match-${EXEC_MODE}"
EXEC_ARGS=data_10KB.txt

echo 'LLFI Profiling...'
echo '---------------------------------------------------------------'
$LLFI_HOME/bin/profile "${EXEC_BIN}-prof" $EXEC_ARGS
echo '---------------------------------------------------------------'
echo 'LLFI profiling done'
echo

echo 'LLFI FI runs...'
echo '---------------------------------------------------------------'
$LLFI_HOME/bin/injectfault "${EXEC_BIN}-fi" $EXEC_ARGS
echo '---------------------------------------------------------------'
echo 'LLFI FI runs done'
echo

echo 'LLFI trace analysis...'
echo '---------------------------------------------------------------'
pushd llfi/llfi_stat_output > /dev/null
# $LLFI_HOME/tools/tracetodot
echo 'Skipped right now as we do not have the right trace format yet'
popd > /dev/null
echo '---------------------------------------------------------------'
echo 'LLFI trace analysis done'
echo
