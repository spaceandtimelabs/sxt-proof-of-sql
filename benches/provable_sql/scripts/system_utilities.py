import os
import subprocess


def run_process(cmd):
    proc = subprocess.Popen(
        [
            cmd,
        ],
        stdout=subprocess.PIPE,
        shell=True,
    )
    (output, err) = proc.communicate()

    if err:
        print(err)

        exit(1)

    return output.decode("utf-8")


def build_binary(force_build):
    binary_file = "target/release/provable_sql"

    if force_build or (os.path.exists(binary_file) is False):
        run_process("cargo build --release --package provable_sql --features valgrind")

        print("Build was successfull!")

    return binary_file
