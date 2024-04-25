#!/bin/bash
set -eou pipefail

# the root directory where the library will be searched
ROOT_DIR=$1

# the library name to be searched (ex. libproofs.so)
LIB_NAME=$2

readarray -d '' lib_files < <(find ${ROOT_DIR} -type f -name "${LIB_NAME}" -print0)

if [ "${#lib_files[@]}" -eq "1" ]; then
    echo ${lib_files[0]}
elif [ "${#lib_files[@]}" -eq "0" ]; then
    echo "No \"${LIB_NAME}\" file found in the \"${ROOT_DIR}\" directory" >&2  # write error message to stderr
    
    exit 1
else
    echo ${lib_files[0]}
fi
