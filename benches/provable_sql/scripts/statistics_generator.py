import os
import math
import psutil
import platform
from datetime import datetime
from system_utilities import *


def create_statistical_data(statistics_file, all_benches):
    with open(statistics_file, "w") as fs:
        fs.write("architecture:" + platform.machine() + "\n")
        fs.write("platform:" + platform.system() + "\n")
        fs.write("cpu cores: " + str(psutil.cpu_count()) + "\n")
        fs.write(
            "ram (GB): "
            + str(round(psutil.virtual_memory().total / (1024.0**3)))
            + "\n"
        )
        fs.write("timestamp: " + datetime.now().strftime("%m/%d/%Y %H:%M:%S") + "\n\n")

        for bench_idx, bench in enumerate(all_benches):
            fs.write("query index: " + str(bench_idx) + "\n")
            fs.write("query string: " + bench.select_statement() + "\n")
            fs.write("min value in the table: " + str(bench.min_value_table()) + "\n")
            fs.write("max value in the table: " + str(bench.max_value_table()) + "\n")
            fs.write(
                "number of table columns: " + str(bench.num_table_columns()) + "\n"
            )
            fs.write(
                "number of result columns: " + str(bench.num_result_columns()) + "\n"
            )
            fs.write("callgrind file path: " + bench.callgrind_svg_file() + "\n")
            fs.write(
                "callgrind table length: " + str(bench.callgrind_table_length()) + "\n"
            )
            fs.write(
                "callgrind number samples: " + str(bench.num_callgrind_samples()) + "\n"
            )
            fs.write(
                "all plots number samples: " + str(bench.num_plot_samples()) + "\n"
            )
            fs.write(
                "all plots table length: "
                + " ".join([str(l) for l in bench.table_lengths()])
                + "\n"
            )
            fs.write(
                "all plots execution time (ms): "
                + " ".join([str(float(ex)) for ex in bench.execution_times()])
                + "\n\n"
            )


def get_statistical_data(statistics_file, benches):
    all_execution_times = []

    if os.path.isfile(statistics_file):
        with open(statistics_file, "r") as fs:
            fs.readline()  # architecture
            fs.readline()  # platform
            fs.readline()  # cpu cores
            fs.readline()  # ram
            fs.readline()  # timestamp

            for bench_idx, bench in enumerate(benches):
                _ = fs.readline()  # read empty line
                ref_query_idx = int(fs.readline().split(": ")[1][:-1])
                ref_query_string = fs.readline().split(": ")[1][:-1]
                ref_min_value_table = float(fs.readline().split(": ")[1][:-1])
                ref_max_value_table = float(fs.readline().split(": ")[1][:-1])
                ref_number_table_columns = int(fs.readline().split(": ")[1][:-1])
                ref_number_result_columns = int(fs.readline().split(": ")[1][:-1])
                ref_callgrind_file_path_ = fs.readline().split(": ")[1][:-1]
                ref_callgrind_table_length = int(fs.readline().split(": ")[1][:-1])
                ref_callgrind_num_samples = int(fs.readline().split(": ")[1][:-1])
                ref_plots_num_samples = int(fs.readline().split(": ")[1][:-1])
                ref_plots_table_length = [
                    int(val) for val in fs.readline().split(": ")[1][:-1].split(" ")
                ]
                ref_plots_exec_times = [
                    float(ex) for ex in fs.readline().split(": ")[1][:-1].split(" ")
                ]

                # we verify if the reference benchmark params are different from the params
                # that generated the current benchmark. If they are different, we error out
                # given that they should not be different
                if ref_query_idx != bench_idx:
                    raise Exception(
                        "Ref query index %d differ from %d" % (ref_query_idx, bench_idx)
                    )
                elif ref_query_string != bench.select_statement():
                    raise Exception(
                        "Ref Query string `%s` differ from `%s`"
                        % (ref_query_string, bench.select_statement())
                    )
                elif math.fabs(ref_min_value_table - bench.min_value_table()) > 1e-3:
                    raise Exception(
                        "Ref min value table %f differ from %f"
                        % (ref_min_value_table, bench.min_value_table())
                    )
                elif math.fabs(ref_max_value_table - bench.max_value_table()) > 1e-3:
                    raise Exception(
                        "Ref max value table %f differ from %f"
                        % (ref_max_value_table, bench.max_value_table())
                    )
                elif ref_number_table_columns != bench.num_table_columns():
                    raise Exception(
                        "Ref number table columns %d differ from %d"
                        % (ref_number_table_columns, bench.num_table_columns())
                    )
                elif ref_number_result_columns != bench.num_result_columns():
                    raise Exception(
                        "Ref number result columns %d differ from %d"
                        % (ref_number_result_columns, bench.num_result_columns())
                    )
                elif ref_callgrind_table_length != bench.callgrind_table_length():
                    raise Exception(
                        "Ref callgrind table length %d differ from %d"
                        % (ref_callgrind_table_length, bench.callgrind_table_length())
                    )
                elif ref_callgrind_num_samples != bench.num_callgrind_samples():
                    raise Exception(
                        "Ref callgrind number of samples %d differ from %d"
                        % (ref_callgrind_num_samples, bench.num_callgrind_samples())
                    )
                elif ref_plots_num_samples != bench.num_plot_samples():
                    raise Exception(
                        "Ref all plots number of samples %d differ from %d"
                        % (ref_plots_num_samples, bench.num_plot_samples())
                    )
                elif ref_plots_table_length != bench.table_lengths():
                    raise Exception(
                        "Ref all plots table lengths %d differ from %d"
                        % (ref_plots_table_length, bench.table_lengths())
                    )
                elif len(ref_plots_exec_times) != len(bench.table_lengths()):
                    raise Exception(
                        "Ref execution time length %d differ from %d"
                        % (len(ref_plots_exec_times), len(bench.table_lengths()))
                    )

                all_execution_times.append(ref_plots_exec_times)

    return all_execution_times


def generate_and_copy_assets(
    output_dir, ref_statistics_dir, statistics_data_file, tar_tgz_data_file, all_benches
):
    # we need to create a statistics data file with all the information about the current benchmark
    create_statistical_data(output_dir + "/" + statistics_data_file, all_benches)

    # we need to fetch the execution data from a possibly previous benchmark given that
    # the current one could not be specified to be executed. If execution was specified,
    # then get_statistical_data should return the same data from the current benchmark.
    all_execution_times = get_statistical_data(
        output_dir + "/" + statistics_data_file, all_benches
    )

    for count, exec in enumerate(all_execution_times):
        all_benches[count].set_execution_times(exec)

    ref_statistical_file = ref_statistics_dir + "/" + statistics_data_file

    # we also need to fetch the execution data from a reference benchmark used as comparison with the current one,
    # possibly located at ../data/
    ref_all_executions_times = get_statistical_data(ref_statistical_file, all_benches)

    for count, exec in enumerate(ref_all_executions_times):
        all_benches[count].set_ref_execution_times(exec)

    # we need to copy the ref_statistical_files from the original directory to the output_directory
    # since we'll reference this file from the html index, which needs all files to be available
    # under the same directory tree
    run_process(
        "cp " + ref_statistical_file + " " + output_dir + "/ref_" + statistics_data_file
    )

    for count, bench in enumerate(all_benches):
        # we also copy the reference callgrind svg files
        run_process(
            "cp "
            + ref_statistics_dir
            + "/"
            + bench.callgrind_svg_file()
            + " "
            + output_dir
            + "/"
            + bench.ref_callgrind_svg_file()
        )

    files_to_save_as_tgz = ""

    # save necessary files to tgz
    for count, bench in enumerate(all_benches):
        files_to_save_as_tgz += output_dir + bench.callgrind_svg_file() + " "

    files_to_save_as_tgz += output_dir + statistics_data_file + " "

    run_process(
        "tar -cvzf " + output_dir + "/" + tar_tgz_data_file + " " + files_to_save_as_tgz
    )
