import numpy as np
import matplotlib.pyplot as plt

def plot_benchmark(output_dir, bench):
    if len(bench.execution_times()) != len(bench.table_lengths()): return

    fig, ax = plt.subplots()

    plt.xscale("log")

    plt.plot(bench.table_lengths(), bench.execution_times(), '--*', color = 'blue')

    plt.ylabel("Execution Time (ms)")

    plt.xlabel("Table Length")

    for i in range(len(bench.execution_times())):
        table_len, exec_time = bench.table_lengths()[i], bench.execution_times()[i]

        throughput = table_len / exec_time

        # convert throughput from `rows / ms` to `rows / minute`
        throughput = throughput * 1e3 * 60

        ax.annotate('%.2g rows/min' % throughput, (table_len, exec_time))

    plt.title('query %d: %s' % (bench.idx(), bench.select_statement()))

    plt.grid(color = 'green', linestyle = '--', linewidth = 0.5)

    fig.savefig(output_dir + '/' + bench.plot_bench_svg_file(), format='svg', dpi=400)

def generate_all_plots_queries(output_dir, all_benches, all_plots_data_file):
    for bench in all_benches: plot_benchmark(output_dir, bench)

    fig, axs = plt.subplots((len(all_benches) + 1) // 2, 2)
    fig.suptitle('All Benchmark Table Plots')

    for count_query, bench in enumerate(all_benches):
        if len(bench.execution_times()) != len(bench.table_lengths()): continue

        axis = axs[count_query // 2, count_query % 2]

        axis.set_xscale("log")

        axis.plot(bench.table_lengths(), bench.execution_times(), '--*', color = 'blue')

        axis.set_ylabel("Execution Time (ms)")

        axis.set_xlabel("Table Length")

        axis.set_title('Query %d: %s' % (count_query, bench.select_statement()))

        axis.grid(color = 'green', linestyle = '--', linewidth = 0.5)

        _, ymax = axis.get_ylim()

        axis.set_yticks(np.round(np.linspace(0, ymax, 6), 2))

    fig.set_size_inches(14.5, 10.5)
    fig.tight_layout()
    fig.savefig(output_dir + '/' + all_plots_data_file, format='svg', dpi=1000)
